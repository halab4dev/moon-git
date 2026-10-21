// Downloading new commits from the remotes, without touching the working tree.

use std::path::Path;

use crate::error::AppError;
use crate::git::operations::{Operations, run_git_with_progress};

/// `git fetch` from every remote. Only remote-tracking branches
/// (origin/main…) move; local branches and files stay as they are, so this
/// is always safe to run.
///
/// Remote-tracking branches deleted on the server are kept (no --prune);
/// cleaning them up can be added later as an explicit option.
pub fn fetch(
    ops: &Operations,
    op_id: &str,
    repo: &Path,
    on_line: impl FnMut(&str),
) -> Result<(), AppError> {
    // --all: every remote, not only the one of the current branch.
    // --progress: git only prints progress to a terminal unless asked.
    run_git_with_progress(ops, op_id, repo, &["fetch", "--all", "--progress"], on_line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorKind;
    use crate::git::status::read_status;
    use crate::git::test_support::{clone_of, commit_file, git, server_and_clone};
    use tempfile::TempDir;

    #[test]
    fn fetch_updates_ahead_and_behind() {
        let (server, mine) = server_and_clone();
        // Someone else pushes a commit...
        let theirs = clone_of(server.path());
        commit_file(theirs.path(), "theirs.txt");
        git(theirs.path(), &["push", "--quiet", "origin", "main"]);
        // ...while I have one local commit.
        commit_file(mine.path(), "mine.txt");

        // Before fetching, my repo doesn't know about their commit.
        let before = read_status(mine.path()).unwrap().branch;
        assert_eq!((before.ahead, before.behind), (1, 0));

        let ops = Operations::new();
        let mut lines = Vec::new();
        fetch(&ops, "op", mine.path(), |line| lines.push(line.to_string())).unwrap();

        let after = read_status(mine.path()).unwrap().branch;
        assert_eq!(after.name.as_deref(), Some("main"));
        assert_eq!(after.upstream.as_deref(), Some("origin/main"));
        assert_eq!((after.ahead, after.behind), (1, 1));
        // Fetch never changes files: their file is not in my working tree.
        assert!(!mine.path().join("theirs.txt").exists());
        assert!(!lines.is_empty(), "git printed no progress at all");
    }

    #[test]
    fn upstream_deleted_on_server_is_gone() {
        let (server, work) = server_and_clone();
        git(work.path(), &["checkout", "--quiet", "-b", "feature"]);
        git(work.path(), &["push", "--quiet", "-u", "origin", "feature"]);
        git(server.path(), &["branch", "-D", "feature"]);
        // Without --prune the old origin/feature stays; remove it like a prune would.
        git(
            work.path(),
            &["branch", "--quiet", "-d", "-r", "origin/feature"],
        );

        let ops = Operations::new();
        fetch(&ops, "op", work.path(), |_| {}).unwrap();

        let branch = read_status(work.path()).unwrap().branch;
        assert_eq!(branch.upstream.as_deref(), Some("origin/feature"));
        assert!(branch.upstream_gone);
    }

    #[test]
    fn repo_without_remotes_fetches_nothing() {
        let dir = TempDir::new().unwrap();
        git(dir.path(), &["init", "--quiet"]);
        let ops = Operations::new();
        fetch(&ops, "op", dir.path(), |_| {}).unwrap();
    }

    #[test]
    fn unreachable_remote_is_an_error() {
        let (server, work) = server_and_clone();
        drop(server); // deletes the server folder

        let ops = Operations::new();
        let err = fetch(&ops, "op", work.path(), |_| {}).unwrap_err();
        assert_eq!(err.kind, ErrorKind::GitFailed);
        assert!(!err.message.is_empty());
    }
}
