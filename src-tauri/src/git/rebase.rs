// Rebasing the current branch onto another branch, tag or commit (step 22).
// Not interactive: every commit of the branch is replayed, in order, on top
// of the target. The replayed commits are new commits (new ids); the old ones
// disappear from the branch.

use std::path::Path;

use crate::error::{AppError, ErrorKind};
use crate::git::merge::{commit_of, current_branch, merge_in_progress, resolve_source};
use crate::git::runner::run_git;
use crate::git::status::read_status;
use crate::models::RebasePreview;

/// What rebasing the current branch onto `onto` (full ref name or commit
/// id) would do; shown in the confirmation before starting.
pub fn preview(repo: &Path, onto: &str) -> Result<RebasePreview, AppError> {
    let target = resolve_source(repo, onto)?;
    let branch = current_branch(repo)?;
    let onto_commit = commit_of(repo, &target.revision)?;
    let base = run_git(repo, &["merge-base", "HEAD", &onto_commit])
        .map_err(|_| {
            invalid(format!(
                "\"{}\" and \"{branch}\" have no commit in common, so the branch can't be rebased onto it.",
                target.label
            ))
        })?
        .trim()
        .to_string();

    // The commits that get replayed: on the branch, not in the target.
    let range = format!("{onto_commit}..HEAD");
    let commits = count(repo, &[&range])?;
    // Of those, the ones not on any remote branch. The rest were pushed:
    // after the rebase, pushing needs --force (the app never does that).
    let unpushed = count(repo, &[&range, "--not", "--remotes"])?;

    Ok(RebasePreview {
        branch,
        onto: target.label,
        // The branch already starts from the target's newest commit.
        up_to_date: base == onto_commit,
        commits,
        pushed: commits - unpushed,
    })
}

pub fn rebase(repo: &Path, onto: &str) -> Result<(), AppError> {
    let target = resolve_source(repo, onto)?;
    current_branch(repo)?;
    let status = read_status(repo)?;
    if !status.conflicted.is_empty() {
        return Err(invalid(
            "Some files have conflicts. Resolve them before rebasing.".to_string(),
        ));
    }
    if merge_in_progress(repo) {
        return Err(invalid(
            "A merge is in progress. Commit it or abort it before rebasing.".to_string(),
        ));
    }
    if rebase_in_progress(repo) {
        return Err(invalid(
            "A rebase is already in progress. Finish it or abort it first.".to_string(),
        ));
    }
    // git refuses too ("cannot rebase: You have unstaged changes"); this
    // says what to do in the app's words. Untracked files are fine.
    if !status.staged.is_empty() || !status.unstaged.is_empty() {
        return Err(invalid(
            "You have uncommitted changes. Commit or stash them before rebasing.".to_string(),
        ));
    }

    // --no-autostash: never stash behind the user's back, whatever
    // rebase.autoStash says (the check above makes it unnecessary anyway).
    match run_git(
        repo,
        &[
            "rebase",
            "--no-autostash",
            "--end-of-options",
            &target.revision,
        ],
    ) {
        Ok(_) => Ok(()),
        Err(err) => Err(explain_rebase_error(repo, err)),
    }
}

/// A rebase stopped halfway (on conflicts). git keeps its state in the
/// folder .git/rebase-merge (or .git/rebase-apply, used by older git and
/// by `git am`) until it is continued to the end or aborted.
pub fn rebase_in_progress(repo: &Path) -> bool {
    let Ok(output) = run_git(
        repo,
        &[
            "rev-parse",
            "--git-path",
            "rebase-merge",
            "--git-path",
            "rebase-apply",
        ],
    ) else {
        return false;
    };
    output.lines().any(|path| repo.join(path).is_dir())
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

fn explain_rebase_error(repo: &Path, err: AppError) -> AppError {
    // As for merge: the status tells whether it stopped on conflicts.
    if let Ok(status) = read_status(repo) {
        let count = status.conflicted.len();
        if count > 0 {
            return AppError::new(
                ErrorKind::Conflict,
                format!(
                    "The rebase stopped because {count} file(s) have conflicts. Fix them in your \
                     editor, stage them, then run `git rebase --continue` in a terminal (or \
                     `git rebase --abort` to undo the rebase). Continue and Abort in MoonGit \
                     come in the next step."
                ),
            );
        }
    }
    if err.message.contains("would be overwritten") {
        let files: Vec<&str> = err
            .message
            .lines()
            .filter_map(|line| line.strip_prefix('\t'))
            .collect();
        return invalid(format!(
            "The rebase would overwrite these untracked files: {}. Move or delete them \
             first.",
            files.join(", ")
        ));
    }
    err
}

fn invalid(message: String) -> AppError {
    AppError::new(ErrorKind::InvalidInput, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_support::{commit_file, git, server_and_clone};
    use std::fs;
    use tempfile::TempDir;

    // main: init ── m1 ── m2        feature: init ── f
    fn diverged() -> TempDir {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        git(root, &["init", "--quiet", "--initial-branch=main"]);
        git(root, &["config", "user.name", "Test"]);
        git(root, &["config", "user.email", "test@example.com"]);
        commit_file(root, "init.txt");
        git(root, &["switch", "--quiet", "--create", "feature"]);
        commit_file(root, "f.txt");
        git(root, &["switch", "--quiet", "main"]);
        commit_file(root, "m1.txt");
        commit_file(root, "m2.txt");
        dir
    }

    fn subjects(root: &Path) -> String {
        git(root, &["log", "--format=%s", "--reverse"]).replace('\n', " ")
    }

    #[test]
    fn previews_what_a_rebase_would_do() {
        let dir = diverged();
        let root = dir.path();
        let p = preview(root, "refs/heads/feature").unwrap();
        assert_eq!((p.branch.as_str(), p.onto.as_str()), ("main", "feature"));
        assert_eq!((p.commits, p.pushed), (2, 0));
        assert!(!p.up_to_date);

        // main already starts from init.
        let init = git(root, &["rev-parse", "main~2"]);
        assert!(preview(root, init.trim()).unwrap().up_to_date);
    }

    #[test]
    fn counts_commits_that_were_already_pushed() {
        let (_server, work) = server_and_clone();
        let root = work.path();
        commit_file(root, "pushed.txt");
        git(root, &["push", "--quiet"]);
        commit_file(root, "local.txt");
        git(root, &["branch", "other", "main~2"]);
        git(root, &["switch", "--quiet", "other"]);
        commit_file(root, "other.txt");
        git(root, &["switch", "--quiet", "main"]);

        let p = preview(root, "refs/heads/other").unwrap();
        assert_eq!((p.commits, p.pushed), (2, 1));
    }

    #[test]
    fn replays_the_branch_on_top_of_the_target() {
        let dir = diverged();
        let root = dir.path();
        let old_m2 = git(root, &["rev-parse", "HEAD"]);

        rebase(root, "refs/heads/feature").unwrap();

        assert_eq!(subjects(root), "init.txt f.txt m1.txt m2.txt ");
        // New commits: same changes, new ids.
        assert_ne!(git(root, &["rev-parse", "HEAD"]), old_m2);
        assert_eq!(
            read_status(root).unwrap().branch.name.as_deref(),
            Some("main")
        );
        assert!(!rebase_in_progress(root));
    }

    #[test]
    fn refuses_uncommitted_changes_but_not_untracked_files() {
        let dir = diverged();
        let root = dir.path();
        fs::write(root.join("init.txt"), "changed\n").unwrap();
        let err = rebase(root, "refs/heads/feature").unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);

        fs::write(root.join("init.txt"), "init.txt\n").unwrap();
        fs::write(root.join("untracked.txt"), "u\n").unwrap();
        rebase(root, "refs/heads/feature").unwrap();
        assert!(root.join("untracked.txt").exists());
    }

    #[test]
    fn stops_on_conflicts_and_is_detected() {
        let dir = diverged();
        let root = dir.path();
        // Both sides change init.txt.
        fs::write(root.join("init.txt"), "main\n").unwrap();
        git(root, &["commit", "--quiet", "-am", "main side"]);
        git(root, &["switch", "--quiet", "feature"]);
        fs::write(root.join("init.txt"), "feature\n").unwrap();
        git(root, &["commit", "--quiet", "-am", "feature side"]);
        git(root, &["switch", "--quiet", "main"]);

        let err = rebase(root, "refs/heads/feature").unwrap_err();

        assert_eq!(err.kind, ErrorKind::Conflict);
        assert!(rebase_in_progress(root));
        // No second rebase, and no merge, on top.
        let err = rebase(root, "refs/heads/feature").unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
        // Nor a commit from the commit box, even after staging the fix.
        git(root, &["add", "--", "init.txt"]);
        let err = crate::git::commit::commit(root, "x", false, false).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
        git(root, &["rebase", "--abort"]);
        assert!(!rebase_in_progress(root));
    }

    #[test]
    fn accepts_only_existing_refs_and_commit_ids() {
        let dir = diverged();
        let root = dir.path();
        for bad in ["refs/heads/nope", "--root", "HEAD", "feature", "main~1"] {
            assert!(rebase(root, bad).is_err(), "{bad}");
        }
        assert_eq!(subjects(root), "init.txt m1.txt m2.txt ");
    }
}
