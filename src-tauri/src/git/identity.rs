// Who commits are made as, per repository (step 14): `user.name` and
// `user.email` in the repository's own config (.git/config), which wins over
// the user-wide one (~/.gitconfig). E.g. a work email for work repositories,
// a personal one for the rest.

use std::path::Path;

use crate::error::{AppError, ErrorKind};
use crate::git::runner::run_git;
use crate::models::Identity;

pub fn read(repo: &Path) -> Result<Identity, AppError> {
    let get = |scope: &str, key: &str| -> Result<Option<String>, AppError> {
        // --default "": an unset key gives "" instead of failing with exit code 1.
        let out = run_git(repo, &["config", scope, "--default", "", "--get", key])?;
        Ok(Some(out.trim().to_string()).filter(|value| !value.is_empty()))
    };
    // What the repository would use without its own setting: the user's
    // ~/.gitconfig, or else the machine-wide one.
    let inherited = |key: &str| -> Result<Option<String>, AppError> {
        Ok(match get("--global", key)? {
            Some(value) => Some(value),
            None => get("--system", key)?,
        })
    };
    Ok(Identity {
        name: get("--local", "user.name")?,
        email: get("--local", "user.email")?,
        inherited_name: inherited("user.name")?,
        inherited_email: inherited("user.email")?,
    })
}

/// The computer-wide identity alone (no repository needed), for the group
/// dialog: name and email of ~/.gitconfig, or else the machine-wide config.
pub fn read_computer() -> Result<Identity, AppError> {
    // Any existing folder works: only --global / --system are read.
    let identity = read(&std::env::temp_dir())?;
    Ok(Identity {
        name: None,
        email: None,
        ..identity
    })
}

/// `own`: Some((name, email)) = this repository's own identity; None = remove
/// it, so the user-wide one applies again. Only .git/config is changed.
pub fn set(repo: &Path, own: Option<(&str, &str)>) -> Result<Identity, AppError> {
    match own {
        Some((name, email)) => {
            let (name, email) = clean_identity(name, email)?;
            // "--": a value starting with "-" is still a value.
            run_git(repo, &["config", "--local", "--", "user.name", &name])?;
            run_git(repo, &["config", "--local", "--", "user.email", &email])?;
        }
        None => {
            for key in ["user.name", "user.email"] {
                unset(repo, key)?;
            }
        }
    }
    read(repo)
}

/// Checks a name + email (repository or group identity): trimmed copies.
pub(crate) fn clean_identity(name: &str, email: &str) -> Result<(String, String), AppError> {
    let name = check(name, "a name")?;
    let email = check(email, "an email address")?;
    if !email.contains('@') {
        return Err(invalid(format!("\"{email}\" is not an email address.")));
    }
    Ok((name.to_string(), email.to_string()))
}

/// Is a repository's own identity the given one? (Group identity, step 14b.)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LocalIdentityState {
    /// Neither name nor email set in .git/config.
    Unset,
    /// Both set, to exactly these values.
    Same,
    /// Something else (other values, or only one of the two set).
    Different,
}

/// Compares the repository's own name/email (.git/config only) with `name` / `email`.
pub fn compare(repo: &Path, name: &str, email: &str) -> Result<LocalIdentityState, AppError> {
    let own = read(repo)?;
    Ok(match (own.name.as_deref(), own.email.as_deref()) {
        (None, None) => LocalIdentityState::Unset,
        (Some(n), Some(e)) if n == name && e == email => LocalIdentityState::Same,
        _ => LocalIdentityState::Different,
    })
}

/// Writes a group's identity into one of its repositories (step 14b).
/// `all`: also over an identity the repository already has; otherwise only
/// a repository without any (neither name nor email) gets it. A repository
/// that already has exactly these values is left as it is.
///
/// Returns true when .git/config was changed.
pub fn apply_group_identity(
    repo: &Path,
    name: &str,
    email: &str,
    all: bool,
) -> Result<bool, AppError> {
    let write = match compare(repo, name, email)? {
        LocalIdentityState::Unset => true,
        LocalIdentityState::Same => false,
        LocalIdentityState::Different => all,
    };
    if write {
        set(repo, Some((name, email)))?;
    }
    Ok(write)
}

fn unset(repo: &Path, key: &str) -> Result<(), AppError> {
    match run_git(repo, &["config", "--local", "--unset-all", key]) {
        Ok(_) => Ok(()),
        // Exit code 5 = it wasn't set: already the wanted result.
        Err(err) if err.message.contains("exit status: 5") => Ok(()),
        Err(err) => Err(err),
    }
}

/// Trimmed, not empty, one line, and no "<" / ">" (git writes the identity
/// as `Name <email>` into every commit and drops those characters).
fn check<'a>(value: &'a str, what: &str) -> Result<&'a str, AppError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(invalid(format!("Enter {what}.")));
    }
    if value.contains(['\n', '\r', '\0', '<', '>']) {
        return Err(invalid(format!(
            "{} can't contain line breaks, \"<\" or \">\".",
            capitalize(what)
        )));
    }
    Ok(value)
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn invalid(message: String) -> AppError {
    AppError::new(ErrorKind::InvalidInput, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_support::git;
    use tempfile::TempDir;

    fn repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        git(dir.path(), &["init", "--quiet"]);
        dir
    }

    #[test]
    fn sets_and_removes_the_repository_identity() {
        let dir = repo();
        let root = dir.path();
        assert_eq!(read(root).unwrap().name, None);

        let identity = set(root, Some((" Work Me ", "me@work.example"))).unwrap();
        assert_eq!(identity.name.as_deref(), Some("Work Me"));
        assert_eq!(identity.email.as_deref(), Some("me@work.example"));
        // Written to the repository's config, where commits pick it up.
        assert_eq!(
            git(root, &["config", "--local", "user.email"]).trim(),
            "me@work.example"
        );
        git(root, &["commit", "--quiet", "--allow-empty", "-m", "x"]);
        assert_eq!(
            git(root, &["log", "-1", "--format=%an <%ae>"]).trim(),
            "Work Me <me@work.example>"
        );

        let identity = set(root, None).unwrap();
        assert_eq!((identity.name, identity.email), (None, None));
        // Removing it again is fine.
        set(root, None).unwrap();
    }

    #[test]
    fn compares_with_a_group_identity() {
        let dir = repo();
        let root = dir.path();
        let state = |name, email| compare(root, name, email).unwrap();
        assert_eq!(state("Me", "me@x.example"), LocalIdentityState::Unset);
        set(root, Some(("Me", "me@x.example"))).unwrap();
        assert_eq!(state("Me", "me@x.example"), LocalIdentityState::Same);
        assert_eq!(
            state("Me", "other@x.example"),
            LocalIdentityState::Different
        );
        // Only one of the two set: different too (writing would change it).
        git(root, &["config", "--unset", "user.name"]);
        assert_eq!(state("Me", "me@x.example"), LocalIdentityState::Different);
    }

    #[test]
    fn writes_a_group_identity_into_repositories() {
        let (empty, same, other) = (repo(), repo(), repo());
        set(same.path(), Some(("Work Me", "me@work.example"))).unwrap();
        set(other.path(), Some(("Private", "me@home.example"))).unwrap();
        let email = |dir: &TempDir| read(dir.path()).unwrap().email;
        let apply = |dir: &TempDir, all| {
            apply_group_identity(dir.path(), "Work Me", "me@work.example", all).unwrap()
        };

        // "Only repositories without a setting": the one with its own keeps it.
        assert!(apply(&empty, false));
        assert_eq!(email(&empty).as_deref(), Some("me@work.example"));
        assert!(!apply(&other, false));
        assert_eq!(email(&other).as_deref(), Some("me@home.example"));
        // Already the same: nothing to write either way.
        assert!(!apply(&same, true));

        // "Write for all": overwritten.
        assert!(apply(&other, true));
        assert_eq!(email(&other).as_deref(), Some("me@work.example"));
    }

    #[test]
    fn values_starting_with_a_dash_are_values() {
        let dir = repo();
        let identity = set(dir.path(), Some(("-x", "-y@example.com"))).unwrap();
        assert_eq!(identity.name.as_deref(), Some("-x"));
        assert_eq!(identity.email.as_deref(), Some("-y@example.com"));
    }

    #[test]
    fn refuses_bad_values_and_changes_nothing() {
        let dir = repo();
        for (name, email) in [
            ("", "me@example.com"),
            ("Me", " "),
            ("Me", "not-an-email"),
            ("Me <x>", "me@example.com"),
            ("Me", "me@example.com\nuser.name=x"),
        ] {
            let err = set(dir.path(), Some((name, email))).unwrap_err();
            assert_eq!(err.kind, ErrorKind::InvalidInput, "{name:?} {email:?}");
        }
        let identity = read(dir.path()).unwrap();
        assert_eq!((identity.name, identity.email), (None, None));
    }
}
