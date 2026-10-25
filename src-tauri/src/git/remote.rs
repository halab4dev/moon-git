// Remotes of a repo (step 26): the servers it fetches from / pushes to,
// e.g. "origin" (where it was cloned from) and "upstream" (the original of a
// fork). Adding one and changing its URL; fetch, pull and push already use
// all of them.

use std::collections::HashMap;
use std::path::Path;

use crate::error::{AppError, ErrorKind};
use crate::git::push::list_remotes;
use crate::git::runner::run_git;

/// `git remote add <name> <url>`. Nothing is downloaded: the new remote's
/// branches appear after the next Fetch (which fetches every remote).
pub fn add(repo: &Path, name: &str, url: &str) -> Result<(), AppError> {
    let name = check_name(repo, name)?;
    let url = check_url(url)?;
    // Checked here for a clear message; git would say "remote x already exists".
    if list_remotes(repo)?.iter().any(|r| r == name) {
        return Err(invalid(format!(
            "A remote named \"{name}\" already exists."
        )));
    }
    // "--": name and url are never read as options.
    run_git(repo, &["remote", "add", "--", name, url])?;
    Ok(())
}

/// Changes the URL of remote `name` (`git remote set-url`). A separate push
/// URL (`remote.<name>.pushurl`), if one was set by hand, is left as it is.
pub fn set_url(repo: &Path, name: &str, url: &str) -> Result<(), AppError> {
    let url = check_url(url)?;
    if !list_remotes(repo)?.iter().any(|r| r == name) {
        return Err(AppError::new(
            ErrorKind::NotFound,
            format!("There is no remote named \"{name}\" (removed in the meantime?)."),
        ));
    }
    run_git(repo, &["remote", "set-url", "--", name, url])?;
    Ok(())
}

/// The URL of every remote, as written in the config (before any
/// `url.<base>.insteadOf` rewriting, so it's the value to edit).
pub fn urls(repo: &Path) -> Result<HashMap<String, String>, AppError> {
    let mut urls = HashMap::new();
    if list_remotes(repo)?.is_empty() {
        // --get-regexp fails (exit code 1) when nothing matches.
        return Ok(urls);
    }
    // -z: "remote.<name>.url\n<value>\0" per entry. A name may contain dots
    // ("team.a"), so it's whatever lies between "remote." and ".url".
    let output = run_git(
        repo,
        &["config", "-z", "--get-regexp", r"^remote\..*\.url$"],
    )?;
    for entry in output.split('\0') {
        let Some((key, value)) = entry.split_once('\n') else {
            continue;
        };
        let Some(name) = key
            .strip_prefix("remote.")
            .and_then(|rest| rest.strip_suffix(".url"))
        else {
            continue;
        };
        // A remote can have several URLs; the first one is the main one.
        urls.entry(name.to_string())
            .or_insert_with(|| value.to_string());
    }
    Ok(urls)
}

fn check_name<'a>(repo: &Path, name: &'a str) -> Result<&'a str, AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(invalid(
            "Enter a name for the remote, e.g. \"origin\".".to_string(),
        ));
    }
    // git accepts "-x", but later commands like "git push -x main" would read
    // it as an option.
    if name.starts_with('-') {
        return Err(invalid(
            "A remote name cannot start with \"-\".".to_string(),
        ));
    }
    // The remote's branches are stored as refs/remotes/<name>/…, so the name
    // follows the same rules as branch names (git checks it the same way).
    let probe = format!("refs/remotes/{name}/probe");
    run_git(repo, &["check-ref-format", &probe]).map_err(|_| {
        invalid(format!(
            "\"{name}\" is not a valid remote name (no spaces, \"..\", \"~\", \"^\", \":\", \"?\", \"*\" or \"[\")."
        ))
    })?;
    Ok(name)
}

fn check_url(url: &str) -> Result<&str, AppError> {
    let url = url.trim();
    if url.is_empty() {
        return Err(invalid(
            "Enter the URL of the remote, e.g. https://github.com/user/repo.git".to_string(),
        ));
    }
    // A "URL" like "-oProxyCommand=…" could be read as an ssh option.
    if url.starts_with('-') {
        return Err(invalid("A remote URL cannot start with \"-\".".to_string()));
    }
    // It is stored in .git/config, one value per line.
    if url.chars().any(|c| c.is_control()) {
        return Err(invalid(
            "The URL contains a line break or another control character.".to_string(),
        ));
    }
    Ok(url)
}

fn invalid(message: String) -> AppError {
    AppError::new(ErrorKind::InvalidInput, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::branches::list_branches;
    use crate::git::test_support::{git, server_and_clone};

    fn add_err(repo: &Path, name: &str, url: &str) -> AppError {
        add(repo, name, url).unwrap_err()
    }

    #[test]
    fn adds_a_remote_that_fetch_then_fills() {
        let (server, work) = server_and_clone();
        let url = server.path().to_string_lossy().to_string();

        // Spaces around the input are ignored.
        add(work.path(), " upstream ", &format!(" {url} ")).unwrap();

        assert_eq!(
            git(work.path(), &["remote", "get-url", "upstream"]).trim(),
            url
        );
        // Listed right away, without branches until fetched.
        let list = list_branches(work.path()).unwrap();
        let upstream = list.remotes.iter().find(|g| g.name == "upstream").unwrap();
        assert!(upstream.branches.is_empty());

        git(work.path(), &["fetch", "--quiet", "--all"]);
        let list = list_branches(work.path()).unwrap();
        let upstream = list.remotes.iter().find(|g| g.name == "upstream").unwrap();
        assert_eq!(upstream.branches[0].name, "main");
    }

    #[test]
    fn refuses_a_name_that_exists_or_is_invalid() {
        let (_server, work) = server_and_clone();
        let dir = work.path();

        let err = add_err(dir, "origin", "https://example.com/x.git");
        assert_eq!(err.kind, ErrorKind::InvalidInput);
        assert_eq!(err.message, "A remote named \"origin\" already exists.");

        assert!(add_err(dir, "  ", "u").message.starts_with("Enter a name"));
        assert!(
            add_err(dir, "-x", "u")
                .message
                .contains("cannot start with")
        );
        assert!(
            add_err(dir, "bad name", "u")
                .message
                .contains("not a valid remote name")
        );
        assert!(
            add_err(dir, "a..b", "u")
                .message
                .contains("not a valid remote name")
        );
        // Nothing was added.
        assert_eq!(list_remotes(dir).unwrap(), vec!["origin"]);
    }

    #[test]
    fn refuses_a_bad_url() {
        let (_server, work) = server_and_clone();
        let dir = work.path();

        assert!(
            add_err(dir, "fork", " ")
                .message
                .starts_with("Enter the URL")
        );
        assert!(
            add_err(dir, "fork", "-oProxyCommand=x")
                .message
                .contains("cannot start with")
        );
        assert!(
            add_err(dir, "fork", "https://a\nb")
                .message
                .contains("control character")
        );
        assert_eq!(list_remotes(dir).unwrap(), vec!["origin"]);
    }

    #[test]
    fn reads_the_url_of_every_remote() {
        let (server, work) = server_and_clone();
        let dir = work.path();
        add(dir, "team.a", "/path with spaces/repo").unwrap();

        let urls = urls(dir).unwrap();
        assert_eq!(urls["origin"], server.path().to_string_lossy());
        // A dot in the name and spaces in the URL are kept.
        assert_eq!(urls["team.a"], "/path with spaces/repo");
        assert_eq!(urls.len(), 2);
    }

    #[test]
    fn no_remotes_no_urls() {
        let dir = tempfile::TempDir::new().unwrap();
        git(dir.path(), &["init", "--quiet"]);
        assert!(urls(dir.path()).unwrap().is_empty());
    }

    #[test]
    fn changes_the_url() {
        let (_server, work) = server_and_clone();
        let dir = work.path();

        set_url(dir, "origin", " https://example.com/new.git ").unwrap();
        assert_eq!(urls(dir).unwrap()["origin"], "https://example.com/new.git");

        let err = set_url(dir, "nope", "https://example.com/x.git").unwrap_err();
        assert_eq!(err.kind, ErrorKind::NotFound);
        let err = set_url(dir, "origin", "-x").unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
        assert_eq!(urls(dir).unwrap()["origin"], "https://example.com/new.git");
    }

    #[test]
    fn a_name_with_a_slash_is_allowed() {
        let (_server, work) = server_and_clone();
        add(work.path(), "team/main", "https://example.com/x.git").unwrap();
        assert!(
            list_remotes(work.path())
                .unwrap()
                .contains(&"team/main".to_string())
        );
    }
}
