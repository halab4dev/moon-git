// How a repository logs in to its servers (step 15a), both kept in the
// repository's own .git/config, so git in a terminal behaves the same:
//
//   - SSH key: `core.sshCommand = ssh -i '<key>' -o IdentitiesOnly=yes`.
//     Not set = this computer's SSH settings (~/.ssh/config, ssh-agent,
//     ~/.ssh/id_ed25519…), as plain git does.
//   - HTTPS login: `moongit.credentials = system` = MoonGit stays out of the
//     way and git uses this computer's credential helpers (osxkeychain…).
//     Not set = MoonGit's saved accounts first (step 13).

use std::path::Path;

use crate::error::{AppError, ErrorKind};
use crate::git::credential::shell_quote;
use crate::git::runner::run_git;
use crate::models::RemoteAccess;

const SSH_KEY: &str = "core.sshCommand";
const CREDENTIALS: &str = "moongit.credentials";
/// What MoonGit writes before and after the quoted key path.
const SSH_PREFIX: &str = "ssh -i ";
const SSH_SUFFIX: &str = " -o IdentitiesOnly=yes";

pub fn read(repo: &Path) -> Result<RemoteAccess, AppError> {
    let ssh_command = get_local(repo, SSH_KEY)?;
    let ssh_key = ssh_command.as_deref().and_then(key_in_command);
    let (ssh_remotes, https_remotes) = remotes_by_protocol(repo)?;
    Ok(RemoteAccess {
        // A command MoonGit didn't write (set by hand): shown, kept unless replaced.
        custom_ssh_command: if ssh_key.is_none() { ssh_command } else { None },
        ssh_key,
        system_credentials: get_local(repo, CREDENTIALS)?.as_deref() == Some("system"),
        ssh_remotes,
        https_remotes,
    })
}

/// `ssh_key`: the private key file to use; None = this computer's SSH
/// settings. `system_credentials`: HTTPS logins from this computer's
/// credential helpers instead of MoonGit's accounts.
pub fn set(
    repo: &Path,
    ssh_key: Option<&str>,
    system_credentials: bool,
) -> Result<RemoteAccess, AppError> {
    match ssh_key {
        Some(key) => {
            let key = check_key(key)?;
            let command = format!("{SSH_PREFIX}{}{SSH_SUFFIX}", shell_quote(&key));
            run_git(repo, &["config", "--local", "--", SSH_KEY, &command])?;
        }
        None => unset(repo, SSH_KEY)?,
    }
    if system_credentials {
        run_git(repo, &["config", "--local", "--", CREDENTIALS, "system"])?;
    } else {
        unset(repo, CREDENTIALS)?;
    }
    read(repo)
}

/// True when the repository chose this computer's credential helpers for
/// HTTPS (`moongit.credentials = system`): MoonGit then adds no helper of its own.
pub fn uses_system_credentials(repo: &Path) -> bool {
    get_local(repo, CREDENTIALS).ok().flatten().as_deref() == Some("system")
}

/// Which remotes use SSH and which HTTPS, from their URL and push URL
/// ("origin", or "origin (push)" when only the push URL uses it).
fn remotes_by_protocol(repo: &Path) -> Result<(Vec<String>, Vec<String>), AppError> {
    let (mut ssh, mut https) = (Vec::new(), Vec::new());
    // Exit code 1 = no remote at all.
    let Ok(output) = run_git(
        repo,
        &[
            "config",
            "-z",
            "--get-regexp",
            r"^remote\..*\.(url|pushurl)$",
        ],
    ) else {
        return Ok((ssh, https));
    };
    for entry in output.split('\0') {
        let Some((key, url)) = entry.split_once('\n') else {
            continue;
        };
        let Some(rest) = key.strip_prefix("remote.") else {
            continue;
        };
        let label = match (rest.strip_suffix(".pushurl"), rest.strip_suffix(".url")) {
            (Some(name), _) => format!("{name} (push)"),
            (None, Some(name)) => name.to_string(),
            (None, None) => continue,
        };
        let list = match protocol(url) {
            Protocol::Ssh => &mut ssh,
            Protocol::Https => &mut https,
            Protocol::Other => continue,
        };
        // "origin" already listed: its push URL adds nothing.
        let base = label.trim_end_matches(" (push)").to_string();
        if !list.contains(&label) && !list.contains(&base) {
            list.push(label);
        }
    }
    Ok((ssh, https))
}

#[derive(Debug, PartialEq)]
enum Protocol {
    Ssh,
    Https,
    /// Local folders, file://, git://…: no login.
    Other,
}

/// "https://…" / "http://…" → Https; "ssh://…" or "git@host:path" → Ssh.
fn protocol(url: &str) -> Protocol {
    let lower = url.to_ascii_lowercase();
    if lower.starts_with("https://") || lower.starts_with("http://") {
        return Protocol::Https;
    }
    if lower.starts_with("ssh://") || lower.starts_with("git+ssh://") {
        return Protocol::Ssh;
    }
    // scp-like "[user@]host:path": a ":" before any "/", and no "://".
    match url.split_once(':') {
        Some((host, _))
            if !url.contains("://")
                && !host.is_empty()
                && !host.contains('/')
                && host.len() > 1 =>
        {
            Protocol::Ssh
        }
        _ => Protocol::Other,
    }
}

/// The key path in a command MoonGit wrote; None for any other command.
fn key_in_command(command: &str) -> Option<String> {
    let quoted = command.strip_prefix(SSH_PREFIX)?.strip_suffix(SSH_SUFFIX)?;
    let inner = quoted.strip_prefix('\'')?.strip_suffix('\'')?;
    // Undo shell_quote: '\'' stands for one quote.
    let key = inner.replace(r"'\''", "'");
    // Anything else still containing a quote wasn't written by shell_quote.
    (!key.is_empty() && shell_quote(&key) == quoted).then_some(key)
}

/// An existing file, and the private key rather than its ".pub" half.
fn check_key(key: &str) -> Result<String, AppError> {
    let key = key.trim();
    if key.is_empty() {
        return Err(invalid("Choose the private key file.".to_string()));
    }
    if key.contains(['\n', '\r', '\0']) {
        return Err(invalid("The key path contains a line break.".to_string()));
    }
    // "~/.ssh/id_work" typed by hand: written out in full.
    let expanded;
    let key = match (key.strip_prefix("~/"), std::env::var_os("HOME")) {
        (Some(rest), Some(home)) => {
            expanded = Path::new(&home).join(rest).to_string_lossy().to_string();
            expanded.as_str()
        }
        _ => key,
    };
    if key.ends_with(".pub") {
        return Err(invalid(format!(
            "{key} is the public half of the key. Choose the file without \".pub\"."
        )));
    }
    if !Path::new(key).is_file() {
        return Err(invalid(format!("There is no file at {key}.")));
    }
    Ok(key.to_string())
}

fn get_local(repo: &Path, key: &str) -> Result<Option<String>, AppError> {
    // --default "": an unset key gives "" instead of failing with exit code 1.
    let out = run_git(repo, &["config", "--local", "--default", "", "--get", key])?;
    Ok(Some(out.trim().to_string()).filter(|value| !value.is_empty()))
}

fn unset(repo: &Path, key: &str) -> Result<(), AppError> {
    match run_git(repo, &["config", "--local", "--unset-all", key]) {
        Ok(_) => Ok(()),
        // Exit code 5 = it wasn't set: already the wanted result.
        Err(err) if err.message.contains("exit status: 5") => Ok(()),
        Err(err) => Err(err),
    }
}

fn invalid(message: String) -> AppError {
    AppError::new(ErrorKind::InvalidInput, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_support::git;
    use std::fs;
    use tempfile::TempDir;

    fn repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        git(dir.path(), &["init", "--quiet"]);
        dir
    }

    /// A key file in a folder with a space and a quote in its name.
    fn key_file(dir: &Path) -> String {
        let folder = dir.join("my keys's");
        fs::create_dir(&folder).unwrap();
        let key = folder.join("id_work");
        fs::write(&key, "not a real key").unwrap();
        key.to_string_lossy().to_string()
    }

    #[test]
    fn sets_and_removes_the_ssh_key() {
        let dir = repo();
        let keys = TempDir::new().unwrap();
        let key = key_file(keys.path());
        assert_eq!(read(dir.path()).unwrap().ssh_key, None);

        let access = set(dir.path(), Some(&key), false).unwrap();
        assert_eq!(access.ssh_key.as_deref(), Some(key.as_str()));
        // Written for git, quoted for the shell git runs it with.
        let command = git(dir.path(), &["config", "--local", "core.sshCommand"]);
        assert_eq!(
            command.trim(),
            format!("ssh -i {} -o IdentitiesOnly=yes", shell_quote(&key))
        );

        // "Use this computer's SSH settings": removed from .git/config.
        let access = set(dir.path(), None, false).unwrap();
        assert_eq!(access.ssh_key, None);
        assert_eq!(
            git(
                dir.path(),
                &[
                    "config",
                    "--local",
                    "--default",
                    "",
                    "--get",
                    "core.sshCommand"
                ]
            )
            .trim(),
            ""
        );
    }

    #[test]
    fn a_command_set_by_hand_is_shown_as_custom() {
        let dir = repo();
        git(
            dir.path(),
            &["config", "core.sshCommand", "ssh -F /some/config"],
        );
        let access = read(dir.path()).unwrap();
        assert_eq!(access.ssh_key, None);
        assert_eq!(
            access.custom_ssh_command.as_deref(),
            Some("ssh -F /some/config")
        );
    }

    #[test]
    fn refuses_a_missing_or_public_key() {
        let dir = repo();
        let keys = TempDir::new().unwrap();
        let public = keys.path().join("id_work.pub");
        fs::write(&public, "ssh-ed25519 AAAA").unwrap();
        for key in ["", "/no/such/key", public.to_str().unwrap()] {
            let err = set(dir.path(), Some(key), false).unwrap_err();
            assert_eq!(err.kind, ErrorKind::InvalidInput, "{key}");
        }
        assert_eq!(read(dir.path()).unwrap().ssh_key, None);
    }

    #[test]
    fn a_tilde_means_the_home_folder() {
        let Some(home) = std::env::var_os("HOME") else {
            return;
        };
        // Not a real key: only checks that "~/" is expanded before looking.
        let err = check_key("~/surely-no-such-key-for-moongit").unwrap_err();
        let expected = Path::new(&home).join("surely-no-such-key-for-moongit");
        assert!(
            err.message.contains(&*expected.to_string_lossy()),
            "{}",
            err.message
        );
    }

    #[test]
    fn system_credentials_switch() {
        let dir = repo();
        assert!(!uses_system_credentials(dir.path()));
        assert!(set(dir.path(), None, true).unwrap().system_credentials);
        assert!(uses_system_credentials(dir.path()));
        assert!(!set(dir.path(), None, false).unwrap().system_credentials);
        assert!(!uses_system_credentials(dir.path()));
    }

    #[test]
    fn lists_the_remotes_of_each_protocol() {
        let dir = repo();
        let root = dir.path();
        git(
            root,
            &[
                "remote",
                "add",
                "origin",
                "git@halab4dev.github.com:halab4dev/moon-git.git",
            ],
        );
        git(
            root,
            &["remote", "add", "upstream", "https://github.com/goc/x.git"],
        );
        git(root, &["remote", "add", "local", "/Users/me/repos/x"]);
        // origin also pushes over HTTPS.
        git(
            root,
            &[
                "config",
                "remote.origin.pushurl",
                "https://github.com/halab4dev/moon-git.git",
            ],
        );

        let access = read(root).unwrap();
        assert_eq!(access.ssh_remotes, ["origin"]);
        assert_eq!(access.https_remotes, ["origin (push)", "upstream"]);
    }

    #[test]
    fn recognises_url_kinds() {
        assert_eq!(protocol("git@github.com:me/x.git"), Protocol::Ssh);
        assert_eq!(protocol("ssh://git@host:2222/x.git"), Protocol::Ssh);
        assert_eq!(protocol("HTTPS://github.com/me/x"), Protocol::Https);
        assert_eq!(protocol("/Users/me/x"), Protocol::Other);
        assert_eq!(protocol("file:///Users/me/x"), Protocol::Other);
        assert_eq!(protocol("git://host/x"), Protocol::Other);
        // A Windows drive letter is a path, not "host:path".
        assert_eq!(protocol("C:\\repos\\x"), Protocol::Other);
    }

    #[test]
    fn reads_back_only_what_it_wrote() {
        assert_eq!(
            key_in_command(&format!(
                "ssh -i {} -o IdentitiesOnly=yes",
                shell_quote("/a b/it's")
            ))
            .as_deref(),
            Some("/a b/it's")
        );
        assert_eq!(
            key_in_command("ssh -i /unquoted -o IdentitiesOnly=yes"),
            None
        );
        assert_eq!(key_in_command("ssh -i '/x' -o Other=yes"), None);
    }
}
