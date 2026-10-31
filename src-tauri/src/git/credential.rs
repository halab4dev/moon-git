// MoonGit as a git credential helper (step 12).
//
// When git needs a username / password for an HTTPS server, it asks its
// "credential helpers" one after the other. For every git command that talks
// to a server, MoonGit puts itself first in that list, for that command only
// (`-c` options, nothing is written to any git config file):
//
//   git -c credential.helper=                       ← forget the configured list…
//       -c credential.helper=!'<MoonGit>' --git-credential   ← …ask MoonGit first
//       -c credential.helper=osxkeychain            ← then the user's own helpers, as before
//       fetch …
//
// git then runs `<MoonGit> --git-credential get` and writes the request on
// its stdin ("protocol=https", "host=github.com", …). MoonGit starts in a
// small helper mode instead of opening a window (see main.rs), looks for a
// saved account and prints "username=…" / "password=…" to stdout, or nothing
// if it has none (git then asks the next helper).
//
// The password only travels through that pipe: never in an argument, an
// environment variable, a file or a log.
//
// Step 12 was the plumbing; step 13a adds the saved accounts (accounts.json
// + keyring, see storage/accounts.rs).

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::error::AppError;
use crate::git::access;
use crate::git::remote;
use crate::git::runner::run_git;
use crate::storage::accounts::{Keyring, SavedAccounts, read_accounts};

/// The argument that starts MoonGit in helper mode instead of opening a window.
pub const HELPER_ARG: &str = "--git-credential";

/// Environment variable telling the helper process where accounts.json is.
/// Only a folder path: the token itself is read from the keyring.
pub const DATA_DIR_ENV: &str = "MOONGIT_DATA_DIR";

static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Called once at start (lib.rs) with the app data folder.
pub fn set_data_dir(dir: PathBuf) {
    let _ = DATA_DIR.set(dir);
}

/// None in tests and before the app has started: then no saved accounts are offered.
pub fn data_dir() -> Option<&'static Path> {
    DATA_DIR.get().map(PathBuf::as_path)
}

/// What git asks for ("protocol=https", "host=github.com", "path=…", "username=…").
#[derive(Debug, Default, PartialEq)]
pub struct CredentialRequest {
    pub protocol: String,
    /// "github.com", or "host:port".
    pub host: String,
    /// Only sent when `credential.useHttpPath` is on.
    pub path: Option<String>,
    /// Set when the URL contains a user name ("https://me@github.com/…").
    pub username: Option<String>,
}

pub struct Credential {
    pub username: String,
    pub password: String,
}

/// Where the helper looks for a saved login. Step 13 adds the real one
/// (accounts + keyring); tests use a fake.
pub trait CredentialSource {
    fn find(&self, request: &CredentialRequest) -> Option<Credential>;
}

/// No saved accounts: the helper never answers (git started outside MoonGit).
pub struct NoSavedAccounts;

impl CredentialSource for NoSavedAccounts {
    fn find(&self, _request: &CredentialRequest) -> Option<Credential> {
        None
    }
}

// ---- the helper side (git → MoonGit) ----

/// Entry point of helper mode: `moongit --git-credential <get|store|erase>`.
/// Returns the process exit code. Never prints anything but the answer.
pub fn helper_main(args: &[String]) -> i32 {
    let operation = args
        .iter()
        .position(|a| a == HELPER_ARG)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
        .unwrap_or("");
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        return 1;
    }
    // Started by git from a MoonGit command: the variable says where the accounts are.
    let answer = match std::env::var_os(DATA_DIR_ENV) {
        Some(dir) => answer(
            operation,
            &input,
            &SavedAccounts {
                dir: PathBuf::from(dir),
                secrets: Keyring,
            },
        ),
        None => answer(operation, &input, &NoSavedAccounts),
    };
    if io::stdout().write_all(answer.as_bytes()).is_err() {
        return 1;
    }
    0
}

/// The helper's reply to one request from git ("" = no answer).
pub fn answer(operation: &str, input: &str, source: &dyn CredentialSource) -> String {
    // "store" (the login worked) and "erase" (the server refused it) are
    // only notices; step 13 decides whether to use them.
    if operation != "get" {
        return String::new();
    }
    let request = parse_request(input);
    if request.host.is_empty() {
        return String::new();
    }
    match source.find(&request) {
        Some(found) if is_safe_value(&found.username) && is_safe_value(&found.password) => {
            format!("username={}\npassword={}\n", found.username, found.password)
        }
        // Nothing saved, or a value that would break the protocol.
        _ => String::new(),
    }
}

/// "key=value" lines, up to an empty line or the end of the input.
fn parse_request(input: &str) -> CredentialRequest {
    let mut request = CredentialRequest::default();
    for line in input.lines() {
        if line.is_empty() {
            break;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "protocol" => request.protocol = value.to_string(),
            "host" => request.host = value.to_string(),
            "path" => request.path = Some(value.to_string()),
            "username" => request.username = Some(value.to_string()),
            _ => {}
        }
    }
    request
}

/// A value with a line break (or NUL) would let it add lines of its own to
/// the answer ("host=evil.com"): never sent. Empty values are useless too.
fn is_safe_value(value: &str) -> bool {
    !value.is_empty() && !value.contains(['\n', '\r', '\0'])
}

/// The server named in git's login errors:
/// "could not read Username for 'https://github.com': terminal prompts disabled",
/// "Authentication failed for 'https://github.com/me/x.git/'" → "github.com".
pub fn server_in_messages(messages: &[String]) -> Option<String> {
    messages.iter().find_map(|message| {
        let (_, rest) = message.split_once(" for '")?;
        let (url, _) = rest.split_once('\'')?;
        http_host(url)
    })
}

/// "https://me@github.com:8443/x.git" → "github.com:8443". None for other
/// kinds of URL (ssh, "git@github.com:x.git", local paths): no login asked there.
pub fn http_host(url: &str) -> Option<String> {
    let (scheme, after_scheme) = url.split_once("://")?;
    if !scheme.eq_ignore_ascii_case("https") && !scheme.eq_ignore_ascii_case("http") {
        return None;
    }
    let authority = after_scheme.split('/').next()?;
    // "me@github.com" → "github.com" (a user name in the URL).
    let host = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);
    (!host.is_empty()).then(|| host.to_string())
}

/// Does MoonGit have an account for a server this command talks to? The
/// servers: URLs among the arguments (clone), and the repository's remotes.
pub fn has_account_for(cwd: &Path, args: &[&str], data_dir: &Path) -> bool {
    let Ok(accounts) = read_accounts(data_dir) else {
        return false;
    };
    if accounts.is_empty() {
        return false;
    }
    let mut hosts: Vec<String> = args.iter().filter_map(|arg| http_host(arg)).collect();
    // Not a repository (clone runs in the parent folder): no remotes, fine.
    if let Ok(urls) = remote::urls(cwd) {
        hosts.extend(urls.values().filter_map(|url| http_host(url)));
    }
    hosts
        .iter()
        .any(|host| accounts.iter().any(|a| a.host.eq_ignore_ascii_case(host)))
}

// ---- the git side (MoonGit → git) ----

/// The `-c` options that put `helper` (normally MoonGit itself) first among
/// the credential helpers. `cwd`: where git will run (its repository config
/// counts too).
///
/// `only_moongit`: MoonGit has an account for this server, so the user's own
/// helpers are left out. After a login git tells *every* helper whether it
/// worked ("store" / "erase"): with a wrong MoonGit token, git would otherwise
/// tell osxkeychain to erase a login that works fine in the terminal.
/// Without a MoonGit account, the user's helpers follow MoonGit as before.
pub fn helper_options(
    cwd: &Path,
    helper: &Path,
    only_moongit: bool,
) -> Result<Vec<String>, AppError> {
    // The repository chose this computer's credentials (step 15a): git's own
    // helpers only, exactly as in a terminal.
    if access::uses_system_credentials(cwd) {
        return Ok(Vec::new());
    }
    // Exit code 1 = no helper configured: an empty list, not an error.
    let configured = if only_moongit {
        String::new()
    } else {
        run_git(cwd, &["config", "--get-all", "credential.helper"]).unwrap_or_default()
    };

    let mut options = vec![
        "-c".to_string(),
        // An empty value clears the list built so far (git 2.9+).
        "credential.helper=".to_string(),
        "-c".to_string(),
        format!(
            "credential.helper=!{} {HELPER_ARG}",
            shell_quote(&helper.to_string_lossy())
        ),
    ];
    for existing in configured.lines().filter(|line| !line.trim().is_empty()) {
        options.push("-c".to_string());
        options.push(format!("credential.helper={existing}"));
    }
    Ok(options)
}

/// git runs a "!…" helper through the shell, so the program path is quoted:
/// 'it'\''s' for a path containing a quote, spaces kept as they are.
pub(crate) fn shell_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', r"'\''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::{Command, Stdio};

    use tempfile::TempDir;

    use crate::git::test_support::git;

    struct Fake;

    impl CredentialSource for Fake {
        fn find(&self, request: &CredentialRequest) -> Option<Credential> {
            match request.host.as_str() {
                "github.com" => Some(Credential {
                    username: "me".to_string(),
                    password: "s3cret".to_string(),
                }),
                "evil.com" => Some(Credential {
                    username: "me".to_string(),
                    password: "x\nhost=other.com".to_string(),
                }),
                _ => None,
            }
        }
    }

    #[test]
    fn answers_get_for_a_known_host() {
        let input = "protocol=https\nhost=github.com\nusername=me\n\n";
        assert_eq!(
            answer("get", input, &Fake),
            "username=me\npassword=s3cret\n"
        );
        assert_eq!(
            parse_request(input),
            CredentialRequest {
                protocol: "https".to_string(),
                host: "github.com".to_string(),
                path: None,
                username: Some("me".to_string()),
            }
        );
    }

    #[test]
    fn no_answer_for_unknown_hosts_other_operations_or_unsafe_values() {
        assert_eq!(
            answer("get", "protocol=https\nhost=gitlab.com\n", &Fake),
            ""
        );
        assert_eq!(
            answer("store", "protocol=https\nhost=github.com\n", &Fake),
            ""
        );
        assert_eq!(
            answer("erase", "protocol=https\nhost=github.com\n", &Fake),
            ""
        );
        assert_eq!(answer("get", "", &Fake), "");
        // A password with a line break could inject its own lines: refused.
        assert_eq!(answer("get", "protocol=https\nhost=evil.com\n", &Fake), "");
        // Step 12: nothing is saved yet.
        assert_eq!(
            answer("get", "protocol=https\nhost=github.com\n", &NoSavedAccounts),
            ""
        );
    }

    #[test]
    fn finds_the_server_in_login_errors() {
        let lines = |text: &str| text.lines().map(str::to_string).collect::<Vec<_>>();
        assert_eq!(
            server_in_messages(&lines(
                "fatal: could not read Username for 'https://github.com': terminal prompts disabled"
            )),
            Some("github.com".to_string())
        );
        assert_eq!(
            server_in_messages(&lines(
                "remote: Invalid username or token.\nfatal: Authentication failed for 'https://me@git.example.com:8443/team/x.git/'"
            )),
            Some("git.example.com:8443".to_string())
        );
        assert_eq!(server_in_messages(&lines("fatal: unable to access")), None);
    }

    #[test]
    fn no_moongit_helper_when_the_repository_uses_the_computers_credentials() {
        let repo = TempDir::new().unwrap();
        git(repo.path(), &["init", "--quiet"]);
        crate::git::access::set(repo.path(), None, true).unwrap();
        assert!(
            helper_options(repo.path(), Path::new("/x/moongit"), false)
                .unwrap()
                .is_empty()
        );
        assert!(
            helper_options(repo.path(), Path::new("/x/moongit"), true)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn http_hosts_only() {
        assert_eq!(
            http_host("https://github.com/me/x.git"),
            Some("github.com".to_string())
        );
        assert_eq!(
            http_host("HTTP://me@host:8080/x"),
            Some("host:8080".to_string())
        );
        assert_eq!(http_host("ssh://git@github.com/me/x.git"), None);
        assert_eq!(http_host("git@github.com:me/x.git"), None);
        assert_eq!(http_host("/Users/me/repo"), None);
    }

    #[test]
    fn knows_when_moongit_has_an_account_for_the_server() {
        use crate::storage::accounts::AccountStore;
        use crate::storage::accounts::tests::MemorySecrets;

        let data = TempDir::new().unwrap();
        let repo = TempDir::new().unwrap();
        git(repo.path(), &["init", "--quiet"]);
        git(
            repo.path(),
            &["remote", "add", "origin", "https://github.com/me/x.git"],
        );

        // No accounts at all.
        assert!(!has_account_for(repo.path(), &["fetch"], data.path()));

        let store = AccountStore::new(
            data.path().to_path_buf(),
            Box::new(MemorySecrets::default()),
        );
        let id = store.begin_sign_in("github.com", "me", "t").unwrap();
        store.finish_sign_in(&id, true).unwrap();
        // The repository's remote is on github.com.
        assert!(has_account_for(repo.path(), &["fetch"], data.path()));
        // Cloning outside any repository: decided by the URL alone.
        let outside = TempDir::new().unwrap();
        assert!(!has_account_for(
            outside.path(),
            &["clone", "--", "https://gitlab.com/me/y.git", "y"],
            data.path()
        ));
        assert!(has_account_for(
            outside.path(),
            &["clone", "--", "https://github.com/me/z.git", "z"],
            data.path()
        ));
    }

    #[test]
    fn quotes_paths_for_the_shell() {
        assert_eq!(
            shell_quote("/Applications/MoonGit.app/x"),
            "'/Applications/MoonGit.app/x'"
        );
        assert_eq!(shell_quote("/a b/it's"), r"'/a b/it'\''s'");
    }

    /// Writes a shell script helper that answers with `user`/`pass` and
    /// records that it was asked.
    fn fake_helper(dir: &Path, name: &str, user: &str, pass: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        let log = dir.join(format!("{name}.log"));
        fs::write(
            &path,
            format!(
                "#!/bin/sh\n[ \"$2\" = get ] || [ \"$1\" = get ] || exit 0\necho asked >> '{}'\nprintf 'username={user}\\npassword={pass}\\n'\n",
                log.display()
            ),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        path
    }

    /// `git credential fill` with our options: what git would use for a login.
    fn fill(repo: &Path, options: &[String]) -> String {
        let mut child = Command::new("git")
            .args(options)
            .args(["credential", "fill"])
            .current_dir(repo)
            .env("GIT_TERMINAL_PROMPT", "0")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(b"protocol=https\nhost=example.com\n\n")
            .unwrap();
        let output = child.wait_with_output().unwrap();
        String::from_utf8_lossy(&output.stdout).to_string()
    }

    #[cfg(unix)]
    #[test]
    fn our_helper_is_asked_first_and_the_users_helpers_stay() {
        let repo = TempDir::new().unwrap();
        git(repo.path(), &["init", "--quiet"]);
        // A folder with a space: the path must survive the shell.
        let bin = TempDir::new().unwrap();
        let bin_dir = bin.path().join("Moon Git");
        fs::create_dir(&bin_dir).unwrap();
        let theirs = fake_helper(&bin_dir, "theirs", "user-helper", "p1");
        let ours = fake_helper(&bin_dir, "ours", "moongit", "p2");
        // The user's own helper, configured in the repository.
        git(
            repo.path(),
            &[
                "config",
                "credential.helper",
                &format!("!{}", shell_quote(&theirs.to_string_lossy())),
            ],
        );

        // Ours answers: git uses it and never asks theirs.
        let options = helper_options(repo.path(), &ours, false).unwrap();
        let filled = fill(repo.path(), &options);
        assert!(filled.contains("username=moongit\npassword=p2"), "{filled}");
        assert!(!bin_dir.join("theirs.log").exists());

        // Ours has nothing (like step 12): the user's helper still works.
        let silent = bin_dir.join("silent");
        fs::write(&silent, "#!/bin/sh\nexit 0\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&silent, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let options = helper_options(repo.path(), &silent, false).unwrap();
        let filled = fill(repo.path(), &options);
        assert!(
            filled.contains("username=user-helper\npassword=p1"),
            "{filled}"
        );

        // MoonGit has an account for the server: the user's helper is left out
        // (git would also send it "erase" if the MoonGit token were wrong).
        let options = helper_options(repo.path(), &silent, true).unwrap();
        assert_eq!(options.len(), 4, "{options:?}");
        assert!(!fill(repo.path(), &options).contains("user-helper"));
    }

    #[test]
    fn no_configured_helper_is_fine() {
        let repo = TempDir::new().unwrap();
        git(repo.path(), &["init", "--quiet"]);
        // Global/system helpers of the machine running the tests may exist:
        // only check that ours comes first, right after the reset.
        let options = helper_options(repo.path(), Path::new("/x/moongit"), false).unwrap();
        assert_eq!(options[0..3], ["-c", "credential.helper=", "-c"]);
        assert_eq!(
            options[3],
            "credential.helper=!'/x/moongit' --git-credential"
        );
    }
}
