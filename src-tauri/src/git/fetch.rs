// Downloading new commits from the remotes, without touching the working tree.

use std::path::Path;

use crate::error::AppError;
use crate::git::operations::{Operations, run_git_with_progress};

/// `git fetch` from every remote. Only remote-tracking branches
/// (origin/main…) move; local branches and files stay as they are, so this
/// is always safe to run.
///
/// --prune (step 36): remote-tracking branches whose branch was deleted on
/// the server (origin/feature/x) are removed: they are only git's copy of
/// the server. Local branches are never touched; one that tracked such a
/// branch shows as "deleted on server" (its upstream is "gone").
pub fn fetch(
    ops: &Operations,
    op_id: &str,
    repo: &Path,
    on_line: impl FnMut(&str),
) -> Result<(), AppError> {
    // --all: every remote, not only the one of the current branch.
    // --progress: git only prints progress to a terminal unless asked.
    run_git_with_progress(
        ops,
        op_id,
        repo,
        &["fetch", "--all", "--prune", "--progress"],
        on_line,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorKind;
    use crate::git::status::read_status;
    use crate::git::test_support::{clone_of, commit_file, git, server_and_clone};
    use tempfile::TempDir;

    #[test]
    fn branches_deleted_on_the_server_are_pruned_but_local_ones_stay() {
        let (_server, mine) = server_and_clone();
        let root = mine.path();
        // A branch on the server, with a local branch tracking it.
        git(root, &["switch", "--quiet", "--create", "feature"]);
        git(root, &["push", "--quiet", "-u", "origin", "feature"]);
        git(root, &["switch", "--quiet", "main"]);
        // Someone deletes it on the server (from another clone).
        let other = clone_of(_server.path());
        git(
            other.path(),
            &["push", "--quiet", "origin", "--delete", "feature"],
        );

        fetch(&Operations::new(), "op", root, |_| {}).unwrap();

        // origin/feature is gone…
        let remote = git(
            root,
            &[
                "for-each-ref",
                "--format=%(refname)",
                "refs/remotes/origin/feature",
            ],
        );
        assert_eq!(remote.trim(), "");
        // …the local branch is still there, its upstream marked as gone.
        let local = git(
            root,
            &[
                "for-each-ref",
                "--format=%(upstream:track)",
                "refs/heads/feature",
            ],
        );
        assert_eq!(local.trim(), "[gone]");
    }

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
