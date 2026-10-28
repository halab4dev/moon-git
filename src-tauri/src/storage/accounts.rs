// Saved logins for HTTPS servers (step 13a).
//
// Two halves, kept apart on purpose:
//   - accounts.json in the app data folder: which accounts exist (id, server,
//     user name). No secret ever goes in this file.
//   - the OS keyring (macOS Keychain, Secret Service on Linux, Windows
//     Credential Manager): the token of each account, under its id.
//
// The app (commands) writes both through AccountStore. The credential helper
// (`moongit --git-credential`, a separate process started by git) only reads
// them, through SavedAccounts.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, ErrorKind};
use crate::git::credential::{Credential, CredentialRequest, CredentialSource};
use crate::models::Account;

const FILE_NAME: &str = "accounts.json";
const FILE_VERSION: u32 = 1;
/// Keyring "service" of every MoonGit token (the account id is the "user").
const KEYRING_SERVICE: &str = "com.github.halab4dev.moongit";

#[derive(Serialize, Deserialize)]
struct AccountFile {
    version: u32,
    accounts: Vec<Account>,
}

// ---- where the tokens live ----

/// The keyring, behind a trait so tests can use a map in memory instead of
/// the real Keychain.
pub trait SecretStore: Send + Sync {
    /// None: no token saved under this id.
    fn get(&self, id: &str) -> Result<Option<String>, AppError>;
    fn set(&self, id: &str, secret: &str) -> Result<(), AppError>;
    fn delete(&self, id: &str) -> Result<(), AppError>;
}

/// The operating system's keyring (crate `keyring`).
pub struct Keyring;

impl Keyring {
    fn entry(id: &str) -> Result<keyring::Entry, AppError> {
        keyring::Entry::new(KEYRING_SERVICE, id).map_err(keyring_error)
    }
}

impl SecretStore for Keyring {
    fn get(&self, id: &str) -> Result<Option<String>, AppError> {
        match Keyring::entry(id)?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(err) => Err(keyring_error(err)),
        }
    }

    fn set(&self, id: &str, secret: &str) -> Result<(), AppError> {
        Keyring::entry(id)?
            .set_password(secret)
            .map_err(keyring_error)
    }

    fn delete(&self, id: &str) -> Result<(), AppError> {
        match Keyring::entry(id)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(keyring_error(err)),
        }
    }
}

// The keyring's error text never contains the secret itself.
fn keyring_error(err: keyring::Error) -> AppError {
    AppError::new(
        ErrorKind::Storage,
        format!("Could not use the system keychain: {err}"),
    )
}

// ---- the app side: create / update accounts ----

/// What to undo if the server refuses a new login (see `finish_sign_in`).
enum Pending {
    /// A new account: remove it again.
    New,
    /// A new token for an existing account: put the previous one back.
    Replaced(String),
}

/// The accounts, for the app's commands. Tauri shares one instance.
pub struct AccountStore {
    dir: PathBuf,
    secrets: Box<dyn SecretStore>,
    /// Sign-ins whose token hasn't been tried on the server yet. In memory only.
    pending: Mutex<HashMap<String, Pending>>,
}

impl AccountStore {
    pub fn new(dir: PathBuf, secrets: Box<dyn SecretStore>) -> AccountStore {
        AccountStore {
            dir,
            secrets,
            pending: Mutex::new(HashMap::new()),
        }
    }

    pub fn list(&self) -> Result<Vec<Account>, AppError> {
        read_accounts(&self.dir)
    }

    /// The account git will use for `host` (the first one listed), if any.
    pub fn find(&self, host: &str) -> Result<Option<Account>, AppError> {
        Ok(read_accounts(&self.dir)?
            .into_iter()
            .find(|a| a.host.eq_ignore_ascii_case(host)))
    }

    /// Saves a login typed into the "Sign in" dialog, before it is tried:
    /// git can only use what is saved. The caller retries the git command,
    /// then calls `finish_sign_in` with the result. Returns the account id.
    pub fn begin_sign_in(
        &self,
        host: &str,
        username: &str,
        token: &str,
    ) -> Result<String, AppError> {
        let host = host.trim();
        let username = username.trim();
        check_value(host, "the server")?;
        check_value(username, "a user name")?;
        // The token is kept exactly as typed, except for surrounding spaces from pasting.
        let token = token.trim();
        check_value(token, "a password or token")?;

        let mut accounts = read_accounts(&self.dir)?;
        let existing = accounts
            .iter()
            .position(|a| a.host.eq_ignore_ascii_case(host) && a.username == username);
        let (account, pending) = match existing {
            Some(index) => {
                let account = accounts.remove(index);
                let previous = self.secrets.get(&account.id)?;
                // No previous token (removed from the keychain by hand): like a new one.
                let pending = previous.map_or(Pending::New, Pending::Replaced);
                (account, pending)
            }
            None => (
                Account {
                    id: new_id(),
                    host: host.to_string(),
                    username: username.to_string(),
                },
                Pending::New,
            ),
        };
        let id = account.id.clone();
        // First in the list: the one git uses for this server from now on.
        accounts.insert(0, account);

        self.secrets.set(&id, token)?;
        if let Err(err) = write_accounts(&self.dir, &accounts) {
            // Don't leave a token behind for an account that wasn't saved.
            if matches!(pending, Pending::New) {
                let _ = self.secrets.delete(&id);
            }
            return Err(err);
        }
        self.lock_pending()?.insert(id.clone(), pending);
        Ok(id)
    }

    /// The server accepted the login (`accepted`): keep it. Refused: undo
    /// `begin_sign_in` (remove the new account, or restore the old token).
    pub fn finish_sign_in(&self, id: &str, accepted: bool) -> Result<(), AppError> {
        let Some(pending) = self.lock_pending()?.remove(id) else {
            return Ok(());
        };
        if accepted {
            return Ok(());
        }
        match pending {
            Pending::Replaced(previous) => self.secrets.set(id, &previous),
            Pending::New => {
                let accounts: Vec<Account> = read_accounts(&self.dir)?
                    .into_iter()
                    .filter(|a| a.id != id)
                    .collect();
                write_accounts(&self.dir, &accounts)?;
                self.secrets.delete(id)
            }
        }
    }

    /// Forgets an account: removed from accounts.json and its token from the
    /// keyring. Git will ask again next time (Sign in dialog).
    pub fn remove(&self, id: &str) -> Result<Vec<Account>, AppError> {
        let accounts: Vec<Account> = read_accounts(&self.dir)?
            .into_iter()
            .filter(|a| a.id != id)
            .collect();
        write_accounts(&self.dir, &accounts)?;
        self.secrets.delete(id)?;
        self.lock_pending()?.remove(id);
        Ok(accounts)
    }

    fn lock_pending(&self) -> Result<MutexGuard<'_, HashMap<String, Pending>>, AppError> {
        self.pending.lock().map_err(|_| {
            AppError::new(
                ErrorKind::Storage,
                "The account list is in a bad state. Please restart the app.".to_string(),
            )
        })
    }
}

/// Not empty, and no line break: the helper writes values as "key=value" lines.
fn check_value(value: &str, what: &str) -> Result<(), AppError> {
    if value.is_empty() || value.contains(['\n', '\r', '\0']) {
        return Err(AppError::new(
            ErrorKind::InvalidInput,
            format!("Enter {what} (one line)."),
        ));
    }
    Ok(())
}

// ---- the helper side: answer git ----

/// The saved accounts as a credential source for the helper process.
pub struct SavedAccounts<S: SecretStore> {
    pub dir: PathBuf,
    pub secrets: S,
}

impl<S: SecretStore> CredentialSource for SavedAccounts<S> {
    fn find(&self, request: &CredentialRequest) -> Option<Credential> {
        if request.protocol != "https" && request.protocol != "http" {
            return None;
        }
        let accounts = read_accounts(&self.dir).ok()?;
        let same_host = |a: &&Account| a.host.eq_ignore_ascii_case(&request.host);
        // A user name in the URL ("https://me@github.com/…") picks that account.
        let account = match &request.username {
            Some(name) => accounts
                .iter()
                .filter(same_host)
                .find(|a| &a.username == name),
            None => accounts.iter().find(same_host),
        }?;
        let password = self.secrets.get(&account.id).ok()??;
        Some(Credential {
            username: account.username.clone(),
            password,
        })
    }
}

// ---- accounts.json ----

pub fn read_accounts(dir: &Path) -> Result<Vec<Account>, AppError> {
    let file = dir.join(FILE_NAME);
    let text = match fs::read_to_string(&file) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(storage_error("read", &file, err)),
    };
    let parsed: AccountFile = serde_json::from_str(&text).map_err(|err| {
        AppError::new(
            ErrorKind::Storage,
            format!("{} is not valid: {err}.", file.display()),
        )
    })?;
    Ok(parsed.accounts)
}

fn write_accounts(dir: &Path, accounts: &[Account]) -> Result<(), AppError> {
    let file = dir.join(FILE_NAME);
    fs::create_dir_all(dir).map_err(|err| storage_error("create the folder for", &file, err))?;
    let data = AccountFile {
        version: FILE_VERSION,
        accounts: accounts.to_vec(),
    };
    let text = serde_json::to_string_pretty(&data).map_err(|err| {
        AppError::new(
            ErrorKind::Storage,
            format!("Could not encode the account list: {err}"),
        )
    })?;
    // Temp file + rename, like repos.json: never half written.
    let tmp = file.with_extension("json.tmp");
    fs::write(&tmp, text).map_err(|err| storage_error("write", &tmp, err))?;
    fs::rename(&tmp, &file).map_err(|err| storage_error("replace", &file, err))?;
    Ok(())
}

fn storage_error(action: &str, file: &Path, err: io::Error) -> AppError {
    AppError::new(
        ErrorKind::Storage,
        format!("Could not {action} {}: {err}", file.display()),
    )
}

fn new_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}")
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    /// The keyring in a map. Clones share the same map (like the real keyring
    /// is shared by the app and the helper process).
    #[derive(Clone, Default)]
    pub struct MemorySecrets(pub Arc<Mutex<HashMap<String, String>>>);

    impl SecretStore for MemorySecrets {
        fn get(&self, id: &str) -> Result<Option<String>, AppError> {
            Ok(self.0.lock().unwrap().get(id).cloned())
        }
        fn set(&self, id: &str, secret: &str) -> Result<(), AppError> {
            self.0
                .lock()
                .unwrap()
                .insert(id.to_string(), secret.to_string());
            Ok(())
        }
        fn delete(&self, id: &str) -> Result<(), AppError> {
            self.0.lock().unwrap().remove(id);
            Ok(())
        }
    }

    fn store() -> (TempDir, MemorySecrets, AccountStore) {
        let dir = TempDir::new().unwrap();
        let secrets = MemorySecrets::default();
        let store = AccountStore::new(dir.path().to_path_buf(), Box::new(secrets.clone()));
        (dir, secrets, store)
    }

    fn request(host: &str, username: Option<&str>) -> CredentialRequest {
        CredentialRequest {
            protocol: "https".to_string(),
            host: host.to_string(),
            path: None,
            username: username.map(str::to_string),
        }
    }

    #[test]
    fn accepted_sign_in_is_kept_and_answers_git() {
        let (dir, secrets, store) = store();
        // A line break inside the token is refused (only the ends are trimmed).
        let err = store
            .begin_sign_in("github.com", "me", "ghp\n1")
            .unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
        let id = store
            .begin_sign_in(" github.com ", "me", " ghp_1 ")
            .unwrap();
        store.finish_sign_in(&id, true).unwrap();

        let accounts = store.list().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(
            (accounts[0].host.as_str(), accounts[0].username.as_str()),
            ("github.com", "me")
        );
        assert_eq!(store.find("GitHub.com").unwrap().unwrap().id, id);
        // The token is in the keyring, never in the file.
        assert_eq!(secrets.get(&id).unwrap().as_deref(), Some("ghp_1"));
        let file = fs::read_to_string(dir.path().join(FILE_NAME)).unwrap();
        assert!(!file.contains("ghp_1"), "{file}");

        // The helper process reads the same data.
        let helper = SavedAccounts {
            dir: dir.path().to_path_buf(),
            secrets,
        };
        let found = helper.find(&request("github.com", None)).unwrap();
        assert_eq!(
            (found.username.as_str(), found.password.as_str()),
            ("me", "ghp_1")
        );
        assert!(helper.find(&request("gitlab.com", None)).is_none());
        assert!(
            helper
                .find(&request("github.com", Some("someone-else")))
                .is_none()
        );
    }

    #[test]
    fn refused_new_account_is_removed() {
        let (_dir, secrets, store) = store();
        let id = store.begin_sign_in("github.com", "me", "wrong").unwrap();
        store.finish_sign_in(&id, false).unwrap();
        assert!(store.list().unwrap().is_empty());
        assert_eq!(secrets.get(&id).unwrap(), None);
    }

    #[test]
    fn refused_new_token_restores_the_old_one() {
        let (_dir, secrets, store) = store();
        let id = store.begin_sign_in("github.com", "me", "old").unwrap();
        store.finish_sign_in(&id, true).unwrap();

        let again = store.begin_sign_in("github.com", "me", "new").unwrap();
        assert_eq!(again, id, "same server + user = same account");
        assert_eq!(secrets.get(&id).unwrap().as_deref(), Some("new"));
        store.finish_sign_in(&id, false).unwrap();
        assert_eq!(secrets.get(&id).unwrap().as_deref(), Some("old"));
        assert_eq!(store.list().unwrap().len(), 1);
    }

    #[test]
    fn latest_sign_in_is_used_first_for_its_server() {
        let (dir, secrets, store) = store();
        let work = store.begin_sign_in("github.com", "work", "t-work").unwrap();
        store.finish_sign_in(&work, true).unwrap();
        let me = store.begin_sign_in("github.com", "me", "t-me").unwrap();
        store.finish_sign_in(&me, true).unwrap();

        let helper = SavedAccounts {
            dir: dir.path().to_path_buf(),
            secrets,
        };
        assert_eq!(
            helper.find(&request("github.com", None)).unwrap().username,
            "me"
        );
        // A user name in the URL still picks the other one.
        assert_eq!(
            helper
                .find(&request("github.com", Some("work")))
                .unwrap()
                .password,
            "t-work"
        );
    }

    #[test]
    fn removed_account_is_gone_from_file_and_keyring() {
        let (dir, secrets, store) = store();
        let keep = store.begin_sign_in("gitlab.com", "me", "t1").unwrap();
        store.finish_sign_in(&keep, true).unwrap();
        let gone = store.begin_sign_in("github.com", "me", "t2").unwrap();
        store.finish_sign_in(&gone, true).unwrap();

        let left = store.remove(&gone).unwrap();
        assert_eq!(
            left.iter().map(|a| a.host.as_str()).collect::<Vec<_>>(),
            ["gitlab.com"]
        );
        assert_eq!(secrets.get(&gone).unwrap(), None);
        assert_eq!(secrets.get(&keep).unwrap().as_deref(), Some("t1"));
        let helper = SavedAccounts {
            dir: dir.path().to_path_buf(),
            secrets,
        };
        assert!(helper.find(&request("github.com", None)).is_none());
    }

    #[test]
    fn empty_values_are_refused() {
        let (_dir, _secrets, store) = store();
        for (host, user, token) in [
            ("", "me", "t"),
            ("github.com", " ", "t"),
            ("github.com", "me", ""),
        ] {
            let err = store.begin_sign_in(host, user, token).unwrap_err();
            assert_eq!(err.kind, ErrorKind::InvalidInput);
        }
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn only_http_requests_are_answered() {
        let (dir, secrets, store) = store();
        let id = store.begin_sign_in("example.com", "me", "t").unwrap();
        store.finish_sign_in(&id, true).unwrap();
        let helper = SavedAccounts {
            dir: dir.path().to_path_buf(),
            secrets,
        };
        let mut ssh = request("example.com", None);
        ssh.protocol = "ssh".to_string();
        assert!(helper.find(&ssh).is_none());
    }
}
