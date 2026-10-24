// Cherry-pick (step 23): apply the changes of one commit (from the History
// tab) to the current branch. Two ways, like SmartGit:
// - commit: a new commit with the same changes, message and author;
// - to the working tree (--no-commit): the changes end up staged, to be
//   reviewed and committed (or combined with other changes) by hand.
// If the commit conflicts, git waits halfway (CHERRY_PICK_HEAD exists) until
// the user continues or aborts; only in "commit" mode, see `run`.

use std::path::Path;

use crate::error::{AppError, ErrorKind};
use crate::git::commit::explain_commit_error;
use crate::git::merge::{current_branch, merge_in_progress};
use crate::git::rebase::rebase_in_progress;
use crate::git::runner::{run_git, run_git_without_editor};
use crate::git::show::check_commit_id;
use crate::git::status::read_status;
use crate::models::{CherryPickAction, CherryPickMode};

pub fn cherry_pick(repo: &Path, commit: &str, mode: CherryPickMode) -> Result<(), AppError> {
    check_commit_id(commit)?;
    current_branch(repo)?;
    if !read_status(repo)?.conflicted.is_empty() {
        return Err(invalid(
            "Some files have conflicts. Resolve them before cherry-picking.".to_string(),
        ));
    }
    if merge_in_progress(repo) || rebase_in_progress(repo) || cherry_pick_in_progress(repo) {
        return Err(invalid(
            "A merge, rebase or cherry-pick is in progress. Finish it or abort it first."
                .to_string(),
        ));
    }

    let mut args = vec!["cherry-pick"];
    if mode == CherryPickMode::WorkingTree {
        args.push("--no-commit");
    }
    // A merge commit has two parents, so "its changes" is ambiguous; take
    // them relative to the first parent (what the merge brought into its
    // branch), as the History tab shows them.
    let spec = format!("{commit}^{{commit}}");
    let parents = run_git(repo, &["show", "--no-patch", "--format=%P", &spec])?;
    if parents.split_whitespace().count() > 1 {
        args.extend(["--mainline", "1"]);
    }
    args.extend(["--end-of-options", commit]);

    match run_git(repo, &args) {
        Ok(_) => Ok(()),
        Err(err) => Err(explain_pick_error(repo, mode, err)),
    }
}

/// A cherry-pick waits for Continue or Abort (after conflicts, in commit mode).
pub fn cherry_pick_in_progress(repo: &Path) -> bool {
    run_git(
        repo,
        &["rev-parse", "--quiet", "--verify", "CHERRY_PICK_HEAD"],
    )
    .is_ok()
}

/// Continue (commit the resolved result) or abort the waiting cherry-pick.
pub fn run(repo: &Path, action: CherryPickAction) -> Result<(), AppError> {
    if !cherry_pick_in_progress(repo) {
        return Err(invalid("There is no cherry-pick in progress.".to_string()));
    }
    if action == CherryPickAction::Abort {
        run_git(repo, &["cherry-pick", "--abort"])?;
        return Ok(());
    }
    // Same checks as before continuing a rebase.
    let status = read_status(repo)?;
    if !status.conflicted.is_empty() {
        return Err(invalid(format!(
            "{} file(s) still have conflicts. Double-click each one to resolve it, then continue.",
            status.conflicted.len()
        )));
    }
    if !status.unstaged.is_empty() {
        let files: Vec<&str> = status.unstaged.iter().map(|f| f.path.as_str()).collect();
        return Err(invalid(format!(
            "Stage your changes before continuing (or undo them): {}.",
            files.join(", ")
        )));
    }
    // --continue commits with the original message; no editor (as for rebase).
    match run_git_without_editor(repo, &["cherry-pick", "--continue"]) {
        Ok(_) => Ok(()),
        Err(err) if is_empty_pick(&err) => Err(invalid(
            "After resolving, the cherry-pick changes nothing (the branch already has these \
             changes). Abort it."
                .to_string(),
        )),
        Err(err) => Err(explain_commit_error(err)),
    }
}

fn is_empty_pick(err: &AppError) -> bool {
    err.message.contains("is now empty") || err.message.contains("nothing to commit")
}

fn explain_pick_error(repo: &Path, mode: CherryPickMode, err: AppError) -> AppError {
    if let Ok(status) = read_status(repo) {
        let count = status.conflicted.len();
        if count > 0 {
            let next = match mode {
                CherryPickMode::Commit => {
                    "Double-click a file to resolve it, then Continue. Abort undoes the \
                     cherry-pick."
                }
                // No CHERRY_PICK_HEAD with --no-commit: there is nothing to
                // continue or abort, only files to fix.
                CherryPickMode::WorkingTree => {
                    "Double-click a file to resolve it; the changes stay uncommitted. To undo instead, run `git reset --merge` in a terminal."
                }
            };
            return AppError::new(
                ErrorKind::Conflict,
                format!("The cherry-pick stopped because {count} file(s) have conflicts. {next}"),
            );
        }
    }
    // The branch already has the same changes: git stops with an "empty"
    // cherry-pick waiting. Nothing was changed, so undo that waiting state.
    if is_empty_pick(&err) {
        let _ = run_git(repo, &["cherry-pick", "--abort"]);
        return invalid(
            "The current branch already has the changes of this commit. Nothing was changed."
                .to_string(),
        );
    }
    if err.message.contains("would be overwritten") {
        let files: Vec<&str> = err
            .message
            .lines()
            .filter_map(|line| line.strip_prefix('\t'))
            .collect();
        return invalid(format!(
            "The cherry-pick would overwrite your uncommitted changes to: {}. Commit or stash \
             them first. Nothing was changed.",
            files.join(", ")
        ));
    }
    explain_commit_error(err)
}

fn invalid(message: String) -> AppError {
    AppError::new(ErrorKind::InvalidInput, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_support::{commit_file, git};
    use std::fs;
    use tempfile::TempDir;

    // main: init ── "main a" (a.txt = main)
    // feature: init ── "add f" ── "change a" (a.txt = feature)
    fn repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        git(root, &["init", "--quiet", "--initial-branch=main"]);
        git(root, &["config", "user.name", "Test"]);
        git(root, &["config", "user.email", "test@example.com"]);
        commit_file(root, "a.txt");
        git(root, &["switch", "--quiet", "--create", "feature"]);
        commit_file(root, "f.txt");
        fs::write(root.join("a.txt"), "feature\n").unwrap();
        git(root, &["commit", "--quiet", "-am", "change a"]);
        git(root, &["switch", "--quiet", "main"]);
        fs::write(root.join("a.txt"), "main\n").unwrap();
        git(root, &["commit", "--quiet", "-am", "main a"]);
        dir
    }

    fn id(root: &Path, rev: &str) -> String {
        git(root, &["rev-parse", rev]).trim().to_string()
    }

    fn subject(root: &Path) -> String {
        git(root, &["log", "-1", "--format=%s"]).trim().to_string()
    }

    #[test]
    fn commit_mode_makes_a_new_commit_with_the_same_message() {
        let dir = repo();
        let root = dir.path();
        let add_f = id(root, "feature~1");
        cherry_pick(root, &add_f, CherryPickMode::Commit).unwrap();
        assert_eq!(subject(root), "f.txt");
        assert_ne!(id(root, "HEAD"), add_f);
        assert!(root.join("f.txt").exists());
        assert!(read_status(root).unwrap().staged.is_empty());
    }

    #[test]
    fn working_tree_mode_leaves_the_changes_staged() {
        let dir = repo();
        let root = dir.path();
        let before = id(root, "HEAD");
        cherry_pick(root, &id(root, "feature~1"), CherryPickMode::WorkingTree).unwrap();
        assert_eq!(id(root, "HEAD"), before);
        let status = read_status(root).unwrap();
        assert_eq!(status.staged.len(), 1);
        assert_eq!(status.staged[0].path, "f.txt");
        assert!(!cherry_pick_in_progress(root));
    }

    #[test]
    fn an_already_applied_commit_changes_nothing() {
        let dir = repo();
        let root = dir.path();
        let add_f = id(root, "feature~1");
        cherry_pick(root, &add_f, CherryPickMode::Commit).unwrap();
        let head = id(root, "HEAD");

        let err = cherry_pick(root, &add_f, CherryPickMode::Commit).unwrap_err();

        assert_eq!(err.kind, ErrorKind::InvalidInput);
        assert!(err.message.contains("already has"), "{}", err.message);
        assert!(!cherry_pick_in_progress(root));
        assert_eq!(id(root, "HEAD"), head);
    }

    #[test]
    fn conflicts_wait_for_continue() {
        let dir = repo();
        let root = dir.path();
        let err = cherry_pick(root, &id(root, "feature"), CherryPickMode::Commit).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Conflict);
        assert!(cherry_pick_in_progress(root));
        let state = read_status(root).unwrap().cherry_pick.unwrap();
        assert_eq!(state.commit, id(root, "feature"));
        assert_eq!(state.subject, "change a");

        // Not before the conflict is resolved and staged.
        assert!(run(root, CherryPickAction::Continue).is_err());
        fs::write(root.join("a.txt"), "both\n").unwrap();
        git(root, &["add", "--", "a.txt"]);
        run(root, CherryPickAction::Continue).unwrap();

        assert!(!cherry_pick_in_progress(root));
        assert_eq!(subject(root), "change a");
        assert_eq!(read_status(root).unwrap().cherry_pick, None);
    }

    #[test]
    fn abort_goes_back() {
        let dir = repo();
        let root = dir.path();
        let head = id(root, "HEAD");
        cherry_pick(root, &id(root, "feature"), CherryPickMode::Commit).unwrap_err();
        // No second operation on top.
        let err = cherry_pick(root, &id(root, "feature~1"), CherryPickMode::Commit).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);

        run(root, CherryPickAction::Abort).unwrap();

        assert_eq!(id(root, "HEAD"), head);
        assert_eq!(fs::read_to_string(root.join("a.txt")).unwrap(), "main\n");
        assert!(read_status(root).unwrap().conflicted.is_empty());
    }

    #[test]
    fn merge_commits_are_picked_against_their_first_parent() {
        let dir = repo();
        let root = dir.path();
        // A merge on another branch that brings in f.txt.
        git(
            root,
            &["switch", "--quiet", "--create", "with-merge", "main~1"],
        );
        commit_file(root, "w.txt");
        git(
            root,
            &["merge", "--quiet", "--no-ff", "--no-edit", "feature~1"],
        );
        git(root, &["switch", "--quiet", "main"]);

        cherry_pick(root, &id(root, "with-merge"), CherryPickMode::Commit).unwrap();

        // Relative to the first parent, the merge added f.txt (not w.txt).
        assert!(root.join("f.txt").exists());
        assert!(!root.join("w.txt").exists());
    }

    #[test]
    fn accepts_only_commit_ids() {
        let dir = repo();
        let root = dir.path();
        for bad in ["feature", "HEAD", "--abort", "refs/heads/feature"] {
            assert!(
                cherry_pick(root, bad, CherryPickMode::Commit).is_err(),
                "{bad}"
            );
        }
        assert_eq!(subject(root), "main a");
    }
}
