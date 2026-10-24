// Discarding local changes of files (step 22d), like SmartGit's "Discard":
// each file goes back to how it is in HEAD, staged and unstaged changes alike.
// Files that HEAD does not have (untracked, or newly added) are moved to the
// system Trash instead of being deleted, so a mistake can be undone there.
// (Discarded changes to tracked files cannot be undone: the UI says so.)

use std::fs;
use std::path::Path;

use crate::error::{AppError, ErrorKind};
use crate::git::repo::has_commits;
use crate::git::runner::run_git;
use crate::git::status::read_status;
use crate::models::{ChangeKind, RepoStatus};

// Paths per git call, as in stage.rs.
const BATCH_SIZE: usize = 100;

/// Discards the changes of `paths` (as listed by the status), moving files
/// unknown to HEAD to the Trash.
pub fn discard(repo: &Path, paths: &[String]) -> Result<(), AppError> {
    discard_with(repo, paths, |file| {
        trash::delete(file).map_err(|err| {
            AppError::new(
                ErrorKind::Io,
                format!("Could not move {} to the Trash: {err}", file.display()),
            )
        })
    })
}

/// `remove`: what to do with a file HEAD does not have. The app moves it to
/// the Trash; tests delete it, so they never fill the real Trash.
fn discard_with(
    repo: &Path,
    paths: &[String],
    remove: impl Fn(&Path) -> Result<(), AppError>,
) -> Result<(), AppError> {
    if paths.is_empty() {
        return Err(invalid("No files selected.".to_string()));
    }
    let status = read_status(repo)?;
    let plan = plan(&status, paths, has_commits(repo))?;

    // 1. Forget the new files in the index (their content stays on disk).
    for batch in plan.unindex.chunks(BATCH_SIZE) {
        run_git(
            repo,
            &with_paths(&["rm", "--cached", "--quiet", "--"], batch),
        )?;
    }
    // 2. Files HEAD has: back to HEAD, in the index and on disk.
    for batch in plan.restore.chunks(BATCH_SIZE) {
        run_git(
            repo,
            &with_paths(
                &["restore", "--source=HEAD", "--staged", "--worktree", "--"],
                batch,
            ),
        )?;
    }
    // 3. Files HEAD does not have: to the Trash (if they are still on disk;
    //    an added file may have been deleted since).
    for path in &plan.remove {
        let file = repo.join(path);
        if file.symlink_metadata().is_ok() {
            remove(&file)?;
            remove_empty_folders(repo, &file);
        }
    }
    Ok(())
}

/// Which paths need which treatment.
#[derive(Debug, Default, PartialEq)]
struct Plan {
    /// In the index but not in HEAD: `git rm --cached`.
    unindex: Vec<String>,
    /// In HEAD: `git restore --source=HEAD`.
    restore: Vec<String>,
    /// Not in HEAD: moved to the Trash.
    remove: Vec<String>,
}

// Decides from the status alone, so every path must be one the status lists:
// the frontend can't make us touch any other file.
fn plan(status: &RepoStatus, paths: &[String], has_commits: bool) -> Result<Plan, AppError> {
    let mut plan = Plan::default();
    for path in paths {
        if status.conflicted.iter().any(|f| &f.path == path) {
            return Err(invalid(format!(
                "\"{path}\" has conflicts. Resolving conflicts (keep ours / theirs) comes in a later step."
            )));
        }
        if status.untracked.iter().any(|f| &f.path == path) {
            plan.remove.push(path.clone());
            continue;
        }
        let staged = status.staged.iter().find(|f| &f.path == path);
        let unstaged = status.unstaged.iter().find(|f| &f.path == path);
        if staged.is_none() && unstaged.is_none() {
            return Err(AppError::new(
                ErrorKind::NotFound,
                format!("\"{path}\" has no changes (any more). The list has been reloaded."),
            ));
        }
        // New in the index (added, or the new name of a rename/copy), or
        // marked with `git add -N` (shows as an unstaged "added").
        let new_in_index = staged.is_some_and(|f| {
            matches!(
                f.kind,
                ChangeKind::Added | ChangeKind::Renamed | ChangeKind::Copied
            )
        }) || unstaged.is_some_and(|f| f.kind == ChangeKind::Added);

        if new_in_index || !has_commits {
            plan.unindex.push(path.clone());
            plan.remove.push(path.clone());
        } else {
            plan.restore.push(path.clone());
        }
        // A rename: the old name comes back from HEAD. (A copy left the
        // original untouched, so there is nothing to bring back.)
        if let Some(orig) = staged
            .filter(|f| f.kind == ChangeKind::Renamed)
            .and_then(|f| f.orig_path.clone())
        {
            plan.restore.push(orig);
        }
    }
    Ok(plan)
}

// Moving "new-folder/a.txt" to the Trash leaves "new-folder" empty; git
// doesn't show empty folders, so they would just be clutter. remove_dir only
// succeeds on an empty folder, so a folder with other files stays.
fn remove_empty_folders(repo: &Path, file: &Path) {
    let mut folder = file.parent();
    while let Some(dir) = folder {
        if dir == repo || fs::remove_dir(dir).is_err() {
            break;
        }
        folder = dir.parent();
    }
}

// As in stage.rs: --literal-pathspecs, so "a*b.txt" is that file and not a pattern.
fn with_paths<'a>(command: &[&'a str], paths: &'a [String]) -> Vec<&'a str> {
    let mut args = vec!["--literal-pathspecs"];
    args.extend_from_slice(command);
    args.extend(paths.iter().map(|p| p.as_str()));
    args
}

fn invalid(message: String) -> AppError {
    AppError::new(ErrorKind::InvalidInput, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_support::{commit_file, git};
    use tempfile::TempDir;

    fn repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        git(root, &["init", "--quiet", "--initial-branch=main"]);
        git(root, &["config", "user.name", "Test"]);
        git(root, &["config", "user.email", "test@example.com"]);
        commit_file(root, "a.txt");
        commit_file(root, "b.txt");
        dir
    }

    // Tests delete instead of using the Trash.
    fn run(root: &Path, paths: &[&str]) -> Result<(), AppError> {
        let paths: Vec<String> = paths.iter().map(|p| p.to_string()).collect();
        discard_with(root, &paths, |file| {
            fs::remove_file(file).map_err(|err| AppError::new(ErrorKind::Io, err.to_string()))
        })
    }

    fn read(root: &Path, name: &str) -> String {
        fs::read_to_string(root.join(name)).unwrap()
    }

    fn clean(root: &Path) -> bool {
        let s = read_status(root).unwrap();
        s.staged.is_empty() && s.unstaged.is_empty() && s.untracked.is_empty()
    }

    #[test]
    fn modified_files_go_back_to_head_staged_or_not() {
        let dir = repo();
        let root = dir.path();
        fs::write(root.join("a.txt"), "staged\n").unwrap();
        git(root, &["add", "--", "a.txt"]);
        fs::write(root.join("a.txt"), "staged, then more\n").unwrap();
        fs::write(root.join("b.txt"), "unstaged\n").unwrap();

        run(root, &["a.txt", "b.txt"]).unwrap();

        assert_eq!(read(root, "a.txt"), "a.txt\n");
        assert_eq!(read(root, "b.txt"), "b.txt\n");
        assert!(clean(root));
    }

    #[test]
    fn deleted_files_come_back() {
        let dir = repo();
        let root = dir.path();
        fs::remove_file(root.join("a.txt")).unwrap();
        git(root, &["rm", "--quiet", "--", "b.txt"]);

        run(root, &["a.txt", "b.txt"]).unwrap();

        assert_eq!(read(root, "a.txt"), "a.txt\n");
        assert_eq!(read(root, "b.txt"), "b.txt\n");
        assert!(clean(root));
    }

    #[test]
    fn new_files_are_removed_and_empty_folders_too() {
        let dir = repo();
        let root = dir.path();
        fs::create_dir_all(root.join("new/deep")).unwrap();
        fs::write(root.join("new/deep/u.txt"), "untracked\n").unwrap();
        fs::write(root.join("added.txt"), "added\n").unwrap();
        git(root, &["add", "--", "added.txt"]);
        // A folder that keeps another file stays.
        fs::create_dir(root.join("keep")).unwrap();
        fs::write(root.join("keep/x.txt"), "x\n").unwrap();
        fs::write(root.join("keep/y.txt"), "y\n").unwrap();

        run(root, &["new/deep/u.txt", "added.txt", "keep/x.txt"]).unwrap();

        assert!(!root.join("new").exists());
        assert!(!root.join("added.txt").exists());
        assert!(root.join("keep/y.txt").exists());
        let status = read_status(root).unwrap();
        assert!(status.staged.is_empty());
        assert_eq!(status.untracked.len(), 1);
    }

    #[test]
    fn a_rename_goes_back_to_the_old_name() {
        let dir = repo();
        let root = dir.path();
        git(root, &["mv", "a.txt", "renamed.txt"]);

        run(root, &["renamed.txt"]).unwrap();

        assert!(!root.join("renamed.txt").exists());
        assert_eq!(read(root, "a.txt"), "a.txt\n");
        assert!(clean(root));
    }

    #[test]
    fn works_before_the_first_commit() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        git(root, &["init", "--quiet"]);
        fs::write(root.join("a.txt"), "a\n").unwrap();
        git(root, &["add", "--", "a.txt"]);

        run(root, &["a.txt"]).unwrap();

        assert!(!root.join("a.txt").exists());
        assert!(clean(root));
    }

    #[test]
    fn only_paths_from_the_status_and_no_conflicts() {
        let dir = repo();
        let root = dir.path();
        // Unchanged file, a file outside the repo, a pattern: all refused,
        // nothing touched.
        fs::write(root.join("b.txt"), "keep me\n").unwrap();
        for bad in ["a.txt", "../outside.txt", "*.txt"] {
            let err = run(root, &[bad, "b.txt"]).unwrap_err();
            assert_eq!(err.kind, ErrorKind::NotFound, "{bad}");
        }
        assert_eq!(read(root, "b.txt"), "keep me\n");

        // A conflict.
        git(root, &["switch", "--quiet", "--create", "other"]);
        fs::write(root.join("a.txt"), "other\n").unwrap();
        git(root, &["commit", "--quiet", "-am", "other"]);
        git(root, &["switch", "--quiet", "main"]);
        git(root, &["checkout", "--quiet", "--", "b.txt"]);
        fs::write(root.join("a.txt"), "main\n").unwrap();
        git(root, &["commit", "--quiet", "-am", "main"]);
        let _ = run_git(root, &["merge", "other"]);
        let err = run(root, &["a.txt"]).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
    }
}
