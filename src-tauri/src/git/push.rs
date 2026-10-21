// Uploading local commits to the server. Never forced: if the server has
// commits we don't, git refuses and the user is told to pull first.

use std::path::Path;

use crate::error::{AppError, ErrorKind};
use crate::git::operations::{Operations, run_git_with_progress};
use crate::git::runner::run_git;

/// Pushes the current branch.
///
/// `new_upstream`: None for a branch that already tracks one (plain
/// `git push`). Some(remote) for a branch that was never pushed: it is pushed
/// to `remote` under the same name and tracks it from then on.
pub fn push(
    ops: &Operations,
    op_id: &str,
    repo: &Path,
    new_upstream: Option<&str>,
    on_line: impl FnMut(&str),
) -> Result<(), AppError> {
    match new_upstream {
        None => run_git_with_progress(ops, op_id, repo, &["push", "--progress"], on_line),
        Some(remote) => {
            // Only names git already knows: a value like "--force" can never get through.
            if !list_remotes(repo)?.iter().any(|r| r == remote) {
                return Err(AppError::new(
                    ErrorKind::InvalidInput,
                    format!("There is no remote named \"{remote}\"."),
                ));
            }
            // HEAD = the current branch, pushed to a branch with the same name.
            let args = ["push", "--progress", "--set-upstream", remote, "HEAD"];
            run_git_with_progress(ops, op_id, repo, &args, on_line)
        }
    }
}

/// Names of the configured remotes ("origin", ...), in git's order.
pub fn list_remotes(repo: &Path) -> Result<Vec<String>, AppError> {
    let out = run_git(repo, &["remote"])?;
    Ok(out
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::status::read_status;
    use crate::git::test_support::{clone_of, commit_file, git, server_and_clone};

    fn push_plain(dir: &Path) -> Result<(), AppError> {
        push(&Operations::new(), "op", dir, None, |_| {})
    }

    #[test]
    fn pushes_local_commits() {
        let (server, mine) = server_and_clone();
        commit_file(mine.path(), "mine.txt");
        assert_eq!(read_status(mine.path()).unwrap().branch.ahead, 1);

        push_plain(mine.path()).unwrap();

        assert_eq!(read_status(mine.path()).unwrap().branch.ahead, 0);
        let on_server = git(server.path(), &["log", "-1", "--format=%s", "main"]);
        assert_eq!(on_server.trim(), "mine.txt");
    }

    #[test]
    fn first_push_of_a_branch_sets_its_upstream() {
        let (server, mine) = server_and_clone();
        git(mine.path(), &["checkout", "--quiet", "-b", "feature"]);
        commit_file(mine.path(), "feature.txt");

        push(
            &Operations::new(),
            "op",
            mine.path(),
            Some("origin"),
            |_| {},
        )
        .unwrap();

        let branch = read_status(mine.path()).unwrap().branch;
        assert_eq!(branch.upstream.as_deref(), Some("origin/feature"));
        assert_eq!(branch.ahead, 0);
        git(
            server.path(),
            &["rev-parse", "--verify", "--quiet", "refs/heads/feature"],
        );
    }

    #[test]
    fn rejected_when_server_has_new_commits() {
        let (server, mine) = server_and_clone();
        let theirs = clone_of(server.path());
        commit_file(theirs.path(), "theirs.txt");
        git(theirs.path(), &["push", "--quiet", "origin", "main"]);
        commit_file(mine.path(), "mine.txt");

        let err = push_plain(mine.path()).unwrap_err();
        assert_eq!(err.kind, ErrorKind::PushRejected, "{}", err.message);
        // Nothing was overwritten on the server.
        let on_server = git(server.path(), &["log", "-1", "--format=%s", "main"]);
        assert_eq!(on_server.trim(), "theirs.txt");
    }

    #[test]
    fn unknown_remote_is_refused_before_running_git() {
        let (_server, mine) = server_and_clone();
        for remote in ["upstream", "--force"] {
            let err =
                push(&Operations::new(), "op", mine.path(), Some(remote), |_| {}).unwrap_err();
            assert_eq!(err.kind, ErrorKind::InvalidInput, "{remote}");
        }
    }

    #[test]
    fn lists_remotes() {
        let (_server, mine) = server_and_clone();
        git(mine.path(), &["remote", "add", "backup", "/nowhere"]);
        assert_eq!(list_remotes(mine.path()).unwrap(), vec!["backup", "origin"]);
    }
}
