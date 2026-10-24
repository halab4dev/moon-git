// Moving the current branch to another commit (step 22c), like SmartGit's
// "Reset". Three modes, differing in what happens to the files:
// - soft:  only the branch moves; the changes of the commits left behind
//          stay staged, ready to be committed again (e.g. to squash them);
// - mixed: the branch moves and the index is reset; those changes stay in
//          the working tree, unstaged (git's default);
// - hard:  the branch moves and index + working tree are reset; every
//          uncommitted change is lost (the UI asks twice).

use std::path::Path;

use crate::error::{AppError, ErrorKind};
use crate::git::cherry_pick::cherry_pick_in_progress;
use crate::git::merge::{commit_of, current_branch, merge_in_progress, resolve_source};
use crate::git::rebase::rebase_in_progress;
use crate::git::runner::run_git;
use crate::git::status::read_status;
use crate::models::{ResetMode, ResetPreview};

/// What resetting the current branch to `target` (full ref name or commit
/// id) would do; shown in the reset dialog.
pub fn preview(repo: &Path, target: &str) -> Result<ResetPreview, AppError> {
    let source = resolve_source(repo, target)?;
    let branch = current_branch(repo)?;
    let commit = commit_of(repo, &source.revision)?;
    let status = read_status(repo)?;

    // Commits on the branch now that it no longer contains afterwards.
    let range = format!("{commit}..HEAD");
    let left_behind = count(repo, &[&range])?;
    let unpushed = count(repo, &[&range, "--not", "--remotes"])?;
    Ok(ResetPreview {
        branch,
        target: source.label,
        is_head: commit == commit_of(repo, "HEAD")?,
        left_behind,
        pushed: left_behind - unpushed,
        // Hard would throw these away. Untracked files are never touched.
        changed_files: (status.staged.len() + status.unstaged.len()) as u32,
    })
}

pub fn reset(repo: &Path, target: &str, mode: ResetMode) -> Result<(), AppError> {
    let source = resolve_source(repo, target)?;
    current_branch(repo)?;
    if merge_in_progress(repo) || rebase_in_progress(repo) || cherry_pick_in_progress(repo) {
        return Err(invalid(
            "A merge, rebase or cherry-pick is in progress. Finish it or abort it before resetting.",
        ));
    }
    if !read_status(repo)?.conflicted.is_empty() {
        return Err(invalid(
            "Some files have conflicts. Resolve them before resetting.",
        ));
    }
    // The full commit id, not the ref name: git 2.33's `reset` does not
    // accept --end-of-options, and a 40-character hex id can never be read
    // as an option. The final "--": no file paths follow.
    let commit = commit_of(repo, &source.revision)?;
    let flag = match mode {
        ResetMode::Soft => "--soft",
        ResetMode::Mixed => "--mixed",
        ResetMode::Hard => "--hard",
    };
    run_git(repo, &["reset", "--quiet", flag, &commit, "--"])?;
    Ok(())
}

fn count(repo: &Path, range: &[&str]) -> Result<u32, AppError> {
    let mut args = vec!["rev-list", "--count"];
    args.extend_from_slice(range);
    let out = run_git(repo, &args)?;
    out.trim().parse().map_err(|_| {
        AppError::new(
            ErrorKind::UnexpectedOutput,
            format!("`git rev-list --count` printed {out:?}"),
        )
    })
}

fn invalid(message: &str) -> AppError {
    AppError::new(ErrorKind::InvalidInput, message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_support::{commit_file, git, server_and_clone};
    use std::fs;
    use tempfile::TempDir;

    // main: a ── b ── c, plus an uncommitted change to a.txt and an untracked file.
    fn three_commits() -> TempDir {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        git(root, &["init", "--quiet", "--initial-branch=main"]);
        git(root, &["config", "user.name", "Test"]);
        git(root, &["config", "user.email", "test@example.com"]);
        for name in ["a.txt", "b.txt", "c.txt"] {
            commit_file(root, name);
        }
        fs::write(root.join("a.txt"), "local change\n").unwrap();
        fs::write(root.join("untracked.txt"), "u\n").unwrap();
        dir
    }

    fn first_commit(root: &Path) -> String {
        git(root, &["rev-list", "--max-parents=0", "HEAD"])
            .trim()
            .to_string()
    }

    fn paths(files: &[crate::models::FileChange]) -> Vec<&str> {
        let mut paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        paths.sort();
        paths
    }

    #[test]
    fn previews_what_a_reset_would_do() {
        let dir = three_commits();
        let root = dir.path();
        let p = preview(root, &first_commit(root)).unwrap();
        assert_eq!(p.branch, "main");
        assert_eq!((p.left_behind, p.pushed, p.changed_files), (2, 0, 1));
        assert!(!p.is_head);

        let head = git(root, &["rev-parse", "HEAD"]);
        assert!(preview(root, head.trim()).unwrap().is_head);
    }

    #[test]
    fn counts_pushed_commits_left_behind() {
        let (_server, work) = server_and_clone();
        let root = work.path();
        commit_file(root, "local.txt");
        let first = first_commit(root);
        let p = preview(root, &first).unwrap();
        // first.txt is the target itself; local.txt is not pushed.
        assert_eq!((p.left_behind, p.pushed), (1, 0));

        git(root, &["push", "--quiet"]);
        assert_eq!(preview(root, &first).unwrap().pushed, 1);
    }

    #[test]
    fn soft_keeps_the_changes_staged() {
        let dir = three_commits();
        let root = dir.path();
        reset(root, &first_commit(root), ResetMode::Soft).unwrap();

        let status = read_status(root).unwrap();
        assert_eq!(git(root, &["rev-list", "--count", "HEAD"]).trim(), "1");
        assert_eq!(paths(&status.staged), ["b.txt", "c.txt"]);
        assert_eq!(paths(&status.unstaged), ["a.txt"]);
    }

    #[test]
    fn mixed_keeps_the_changes_unstaged() {
        let dir = three_commits();
        let root = dir.path();
        reset(root, &first_commit(root), ResetMode::Mixed).unwrap();

        let status = read_status(root).unwrap();
        assert!(status.staged.is_empty());
        assert_eq!(paths(&status.unstaged), ["a.txt"]);
        // b.txt and c.txt are no longer in any commit: untracked now.
        assert_eq!(
            paths(&status.untracked),
            ["b.txt", "c.txt", "untracked.txt"]
        );
    }

    #[test]
    fn hard_resets_files_but_keeps_untracked_ones() {
        let dir = three_commits();
        let root = dir.path();
        reset(root, &first_commit(root), ResetMode::Hard).unwrap();

        let status = read_status(root).unwrap();
        assert!(status.staged.is_empty() && status.unstaged.is_empty());
        assert_eq!(fs::read_to_string(root.join("a.txt")).unwrap(), "a.txt\n");
        assert!(!root.join("b.txt").exists());
        assert_eq!(paths(&status.untracked), ["untracked.txt"]);
    }

    #[test]
    fn works_with_branch_names_too() {
        let dir = three_commits();
        let root = dir.path();
        git(root, &["branch", "old", "HEAD~1"]);
        reset(root, "refs/heads/old", ResetMode::Mixed).unwrap();
        assert_eq!(git(root, &["rev-list", "--count", "HEAD"]).trim(), "2");
        assert_eq!(
            read_status(root).unwrap().branch.name.as_deref(),
            Some("main")
        );
    }

    #[test]
    fn accepts_only_existing_refs_and_commit_ids() {
        let dir = three_commits();
        let root = dir.path();
        for bad in ["refs/heads/nope", "--hard", "HEAD", "main", "HEAD~1"] {
            assert!(reset(root, bad, ResetMode::Hard).is_err(), "{bad}");
        }
        assert_eq!(git(root, &["rev-list", "--count", "HEAD"]).trim(), "3");
        assert_eq!(
            fs::read_to_string(root.join("a.txt")).unwrap(),
            "local change\n"
        );
    }
}
