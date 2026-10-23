// Merging another branch, tag or commit into the current branch (step 21).
// Three ways, like SmartGit's merge dialog:
// - fast-forward: only move the branch forward (possible when it has no
//   commits of its own);
// - merge commit: always create one (--no-ff), even when a fast-forward
//   would be possible, so the history shows that a branch was merged;
// - to the working tree: do the merge but stop before committing, so the
//   result can be reviewed (and changed) first. Committing then creates the
//   merge commit.

use std::path::Path;

use crate::error::{AppError, ErrorKind};
use crate::git::commit::explain_commit_error;
use crate::git::refs::list_refs;
use crate::git::runner::run_git;
use crate::git::show::check_commit_id;
use crate::git::status::read_status;
use crate::models::{MergeMode, MergePreview, RefKind};

/// What merging `source` (full ref name or commit id) into the current
/// branch would do; shown in the dialog before choosing.
pub fn preview(repo: &Path, source: &str) -> Result<MergePreview, AppError> {
    let source = resolve_source(repo, source)?;
    let branch = current_branch(repo)?;
    let head = commit_of(repo, "HEAD")?;
    let target = commit_of(repo, &source.revision)?;
    // The newest commit both have. No common commit at all: unrelated histories.
    let base = run_git(repo, &["merge-base", "HEAD", &target])
        .map_err(|_| {
            invalid(format!(
                "\"{}\" and \"{branch}\" have no commit in common, so they can't be merged.",
                source.label
            ))
        })?
        .trim()
        .to_string();
    Ok(MergePreview {
        branch,
        source: source.label,
        // Everything in `source` is already in the branch: nothing to merge.
        up_to_date: base == target,
        // The branch has nothing of its own: it can simply move forward.
        can_fast_forward: base == head,
    })
}

pub fn merge(repo: &Path, source: &str, mode: MergeMode) -> Result<(), AppError> {
    let source = resolve_source(repo, source)?;
    let branch = current_branch(repo)?;
    let status = read_status(repo)?;
    if !status.conflicted.is_empty() {
        return Err(invalid(
            "Some files have conflicts. Resolve them before merging.".to_string(),
        ));
    }
    if merge_in_progress(repo) {
        return Err(invalid(
            "A merge is already in progress. Commit it or abort it first.".to_string(),
        ));
    }

    // Our own message: git's default would say "Merge branch 'refs/heads/x'",
    // because the full ref name is what we pass (to never mix up a branch
    // and a tag with the same name).
    let message = format!("--message=Merge {} into {branch}", source.describe());
    let mut args = vec!["merge"];
    match mode {
        MergeMode::FastForward => args.push("--ff-only"),
        MergeMode::MergeCommit => args.extend(["--no-ff", "--no-edit", &message]),
        // --no-ff too: without it, a possible fast-forward would just happen,
        // and there would be nothing left to review.
        MergeMode::WorkingTree => args.extend(["--no-ff", "--no-commit", &message]),
    }
    args.extend(["--end-of-options", &source.revision]);
    match run_git(repo, &args) {
        Ok(_) => Ok(()),
        Err(err) => Err(explain_merge_error(repo, err)),
    }
}

/// Undoes the merge that waits for its commit: the branch stays where it
/// was, and the merged changes leave the working tree. (The UI asks first.)
pub fn abort(repo: &Path) -> Result<(), AppError> {
    if !merge_in_progress(repo) {
        return Err(invalid("There is no merge in progress.".to_string()));
    }
    run_git(repo, &["merge", "--abort"])?;
    Ok(())
}

/// A merge stopped before its commit: "to the working tree", or because of
/// conflicts. (`MERGE_HEAD` exists until it is committed or aborted.)
pub fn merge_in_progress(repo: &Path) -> bool {
    run_git(repo, &["rev-parse", "--quiet", "--verify", "MERGE_HEAD"]).is_ok()
}

struct Source {
    /// What git is given: a full ref name or a commit id.
    revision: String,
    /// Short name shown to the user: "feature", "origin/feature", "v1.0", "1234abcd".
    label: String,
    kind: Option<RefKind>,
}

impl Source {
    // For the commit message, worded like git's own messages.
    fn describe(&self) -> String {
        let what = match self.kind {
            Some(RefKind::Branch) => "branch",
            Some(RefKind::Remote) => "remote-tracking branch",
            Some(RefKind::Tag) => "tag",
            _ => "commit",
        };
        format!("{what} '{}'", self.label)
    }
}

// Only a branch, remote branch or tag that exists, or a commit id: never
// anything git would read as an option or a revision expression.
fn resolve_source(repo: &Path, source: &str) -> Result<Source, AppError> {
    if source.starts_with("refs/") {
        let refs = list_refs(repo)?;
        let found = refs
            .iter()
            .find(|r| r.full_name == source && r.kind != RefKind::Head)
            .ok_or_else(|| {
                AppError::new(
                    ErrorKind::NotFound,
                    format!("\"{source}\" does not exist (any more)."),
                )
            })?;
        return Ok(Source {
            revision: found.full_name.clone(),
            label: found.name.clone(),
            kind: Some(found.kind),
        });
    }
    check_commit_id(source)?;
    Ok(Source {
        revision: source.to_string(),
        label: source.chars().take(8).collect(),
        kind: None,
    })
}

fn current_branch(repo: &Path) -> Result<String, AppError> {
    read_status(repo)?.branch.name.ok_or_else(|| {
        invalid("HEAD is detached. Check out the branch to merge into first.".to_string())
    })
}

// ^{commit}: for a tag, the commit it points at (not the tag object).
fn commit_of(repo: &Path, revision: &str) -> Result<String, AppError> {
    let spec = format!("{revision}^{{commit}}");
    let out = run_git(
        repo,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            "--end-of-options",
            &spec,
        ],
    )?;
    Ok(out.trim().to_string())
}

fn explain_merge_error(repo: &Path, err: AppError) -> AppError {
    // As for pull and stash: the status tells whether it stopped on conflicts.
    if let Ok(status) = read_status(repo) {
        let count = status.conflicted.len();
        if count > 0 {
            return AppError::new(
                ErrorKind::Conflict,
                format!(
                    "The merge stopped because {count} file(s) have conflicts. Fix them in your \
                     editor, stage them and commit (or run `git merge --abort` in a terminal to \
                     undo the merge). Resolving conflicts inside MoonGit comes in a later step."
                ),
            );
        }
    }
    // "…would be overwritten by merge:\n\ta.txt\n\tb.txt\n…": the files are
    // the lines starting with a tab.
    if err.message.contains("would be overwritten") {
        let files: Vec<&str> = err
            .message
            .lines()
            .filter_map(|line| line.strip_prefix('\t'))
            .collect();
        let files = files.join(", ");
        return invalid(if err.message.contains("untracked working tree files") {
            format!(
                "The merge would overwrite these untracked files: {files}. Move or delete them \
                 first. Nothing was changed."
            )
        } else {
            format!(
                "The merge would overwrite your uncommitted changes to: {files}. Commit or \
                 stash them first. Nothing was changed."
            )
        });
    }
    if err.message.contains("Not possible to fast-forward") {
        return invalid(
            "The branch has commits of its own, so it can't simply move forward. \
             Create a merge commit instead."
                .to_string(),
        );
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

    // main: init ── m        feature: init ── f
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
        commit_file(root, "m.txt");
        dir
    }

    fn parents_of_head(root: &Path) -> usize {
        git(root, &["log", "-1", "--format=%P"])
            .split_whitespace()
            .count()
    }

    fn subject(root: &Path) -> String {
        git(root, &["log", "-1", "--format=%s"]).trim().to_string()
    }

    #[test]
    fn previews_what_a_merge_would_do() {
        let dir = diverged();
        let root = dir.path();
        let p = preview(root, "refs/heads/feature").unwrap();
        assert_eq!((p.branch.as_str(), p.source.as_str()), ("main", "feature"));
        assert!(!p.up_to_date && !p.can_fast_forward);

        // An older commit of main is already in main: nothing to merge.
        let init = git(root, &["rev-parse", "main~1"]);
        assert!(preview(root, init.trim()).unwrap().up_to_date);

        // A branch with nothing of its own can move forward.
        git(root, &["branch", "behind", "main~1"]);
        git(root, &["switch", "--quiet", "behind"]);
        let p = preview(root, "refs/heads/main").unwrap();
        assert!(p.can_fast_forward && !p.up_to_date);
    }

    #[test]
    fn creates_a_merge_commit_with_a_readable_message() {
        let dir = diverged();
        let root = dir.path();
        merge(root, "refs/heads/feature", MergeMode::MergeCommit).unwrap();
        assert_eq!(parents_of_head(root), 2);
        assert_eq!(subject(root), "Merge branch 'feature' into main");
        assert!(!merge_in_progress(root));
    }

    #[test]
    fn merge_commit_even_when_fast_forward_is_possible() {
        let dir = diverged();
        let root = dir.path();
        git(
            root,
            &["switch", "--quiet", "--create", "later", "feature~1"],
        );
        merge(root, "refs/heads/feature", MergeMode::MergeCommit).unwrap();
        assert_eq!(parents_of_head(root), 2);
    }

    #[test]
    fn fast_forward_only_moves_the_branch() {
        let dir = diverged();
        let root = dir.path();
        // On main, feature has diverged: no fast-forward possible.
        let err = merge(root, "refs/heads/feature", MergeMode::FastForward).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);

        git(
            root,
            &["switch", "--quiet", "--create", "later", "feature~1"],
        );
        merge(root, "refs/heads/feature", MergeMode::FastForward).unwrap();
        let head = git(root, &["rev-parse", "HEAD"]);
        let feature = git(root, &["rev-parse", "feature"]);
        assert_eq!(head, feature);
    }

    #[test]
    fn merge_to_working_tree_stops_before_the_commit() {
        let dir = diverged();
        let root = dir.path();
        // A tag, to check its wording in the message too.
        git(root, &["tag", "v1", "feature"]);
        let before = git(root, &["rev-parse", "HEAD"]);

        merge(root, "refs/tags/v1", MergeMode::WorkingTree).unwrap();

        assert_eq!(git(root, &["rev-parse", "HEAD"]), before);
        assert!(merge_in_progress(root));
        let status = read_status(root).unwrap();
        assert_eq!(status.staged.len(), 1);
        assert_eq!(status.staged[0].path, "f.txt");
        // Committing now creates the merge commit, with the prepared message.
        git(root, &["commit", "--quiet", "--no-edit"]);
        assert_eq!(parents_of_head(root), 2);
        assert_eq!(subject(root), "Merge tag 'v1' into main");
    }

    #[test]
    fn conflicts_are_reported_and_left_for_the_user() {
        let dir = diverged();
        let root = dir.path();
        // Both sides change the same line of init.txt.
        fs::write(root.join("init.txt"), "main\n").unwrap();
        git(root, &["commit", "--quiet", "-am", "main side"]);
        git(root, &["switch", "--quiet", "feature"]);
        fs::write(root.join("init.txt"), "feature\n").unwrap();
        git(root, &["commit", "--quiet", "-am", "feature side"]);
        git(root, &["switch", "--quiet", "main"]);

        let err = merge(root, "refs/heads/feature", MergeMode::MergeCommit).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Conflict);
        assert!(merge_in_progress(root));

        // No second merge on top.
        let err = merge(root, "refs/heads/feature", MergeMode::MergeCommit).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
    }

    #[test]
    fn refuses_to_overwrite_uncommitted_changes() {
        let dir = diverged();
        let root = dir.path();
        // feature adds f.txt; an untracked f.txt here would be overwritten.
        fs::write(root.join("f.txt"), "mine\n").unwrap();
        let err = merge(root, "refs/heads/feature", MergeMode::MergeCommit).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
        assert!(
            err.message.contains("untracked files: f.txt"),
            "{}",
            err.message
        );
        assert_eq!(fs::read_to_string(root.join("f.txt")).unwrap(), "mine\n");

        // A tracked file with local changes that the merge touches.
        fs::remove_file(root.join("f.txt")).unwrap();
        git(root, &["switch", "--quiet", "feature"]);
        fs::write(root.join("init.txt"), "feature\n").unwrap();
        git(root, &["commit", "--quiet", "-am", "feature side"]);
        git(root, &["switch", "--quiet", "main"]);
        fs::write(root.join("init.txt"), "wip\n").unwrap();
        let err = merge(root, "refs/heads/feature", MergeMode::MergeCommit).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
        assert!(err.message.contains("init.txt"), "{}", err.message);
        assert_eq!(fs::read_to_string(root.join("init.txt")).unwrap(), "wip\n");
    }

    #[test]
    fn accepts_only_existing_refs_and_commit_ids() {
        let dir = diverged();
        let root = dir.path();
        for bad in ["refs/heads/nope", "--all", "HEAD", "main", "feature~1"] {
            assert!(merge(root, bad, MergeMode::MergeCommit).is_err(), "{bad}");
        }
        assert_eq!(parents_of_head(root), 1);
    }

    #[test]
    fn status_shows_the_merge_with_its_message() {
        let dir = diverged();
        let root = dir.path();
        assert_eq!(read_status(root).unwrap().merge, None);

        merge(root, "refs/heads/feature", MergeMode::WorkingTree).unwrap();

        let state = read_status(root).unwrap().merge.unwrap();
        assert_eq!(state.message, "Merge branch 'feature' into main");
    }

    #[test]
    fn conflict_hints_are_left_out_of_the_message() {
        let dir = diverged();
        let root = dir.path();
        fs::write(root.join("init.txt"), "main\n").unwrap();
        git(root, &["commit", "--quiet", "-am", "main side"]);
        git(root, &["switch", "--quiet", "feature"]);
        fs::write(root.join("init.txt"), "feature\n").unwrap();
        git(root, &["commit", "--quiet", "-am", "feature side"]);
        git(root, &["switch", "--quiet", "main"]);
        merge(root, "refs/heads/feature", MergeMode::MergeCommit).unwrap_err();

        // git's MERGE_MSG ends with "# Conflicts:" and "#\tinit.txt" lines.
        let state = read_status(root).unwrap().merge.unwrap();
        assert_eq!(state.message, "Merge branch 'feature' into main");
    }

    #[test]
    fn abort_undoes_the_merge() {
        let dir = diverged();
        let root = dir.path();
        let before = git(root, &["rev-parse", "HEAD"]);
        merge(root, "refs/heads/feature", MergeMode::WorkingTree).unwrap();

        abort(root).unwrap();

        let status = read_status(root).unwrap();
        assert_eq!(status.merge, None);
        assert!(status.staged.is_empty());
        assert!(!root.join("f.txt").exists());
        assert_eq!(git(root, &["rev-parse", "HEAD"]), before);
        assert_eq!(abort(root).unwrap_err().kind, ErrorKind::InvalidInput);
    }
}
