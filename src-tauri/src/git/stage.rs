// Moving changes between the working tree and the index (staging area).
// Neither operation touches file contents on disk.

use std::path::Path;

use crate::error::{AppError, ErrorKind};
use crate::git::repo::has_commits;
use crate::git::runner::run_git;

// Paths per git call. Thousands of paths in one command line could exceed the
// OS limit (small on Windows), so long lists are sent in batches.
const BATCH_SIZE: usize = 100;

/// `git add` for the given paths: new, modified and deleted files alike.
pub fn stage(repo: &Path, paths: &[String]) -> Result<(), AppError> {
    check_paths(paths)?;
    for batch in paths.chunks(BATCH_SIZE) {
        // --all: also stage deletions of the given paths.
        run_git(repo, &with_paths(&["add", "--all", "--"], batch))?;
    }
    Ok(())
}

/// Takes the given paths out of the index again; the working tree keeps the changes.
/// For a renamed file, pass both the new and the old path.
pub fn unstage(repo: &Path, paths: &[String]) -> Result<(), AppError> {
    check_paths(paths)?;
    // A repository without commits has no HEAD to restore the index from.
    let has_commits = has_commits(repo);

    for batch in paths.chunks(BATCH_SIZE) {
        let command: &[&str] = if has_commits {
            &["restore", "--staged", "--"]
        } else {
            // Before the first commit, "unstage" = forget the file in the index.
            // --cached: keep the file on disk. --ignore-unmatch: paths not in the index are fine.
            &["rm", "--cached", "-r", "--quiet", "--ignore-unmatch", "--"]
        };
        run_git(repo, &with_paths(command, batch))?;
    }
    Ok(())
}

// Builds the argument list. --literal-pathspecs makes git treat each path as
// an exact file name: without it "a*b.txt" would be a pattern and also match "aXb.txt".
fn with_paths<'a>(command: &[&'a str], paths: &'a [String]) -> Vec<&'a str> {
    let mut args = vec!["--literal-pathspecs"];
    args.extend_from_slice(command);
    args.extend(paths.iter().map(|p| p.as_str()));
    args
}

fn check_paths(paths: &[String]) -> Result<(), AppError> {
    if paths.is_empty() || paths.iter().any(|p| p.is_empty()) {
        return Err(AppError::new(
            ErrorKind::InvalidInput,
            "No files selected.".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::status::read_status;
    use crate::models::{ChangeKind, FileChange, RepoStatus};
    use std::fs;
    use tempfile::TempDir;

    fn git(dir: &Path, args: &[&str]) -> String {
        let mut all = vec!["-c", "user.name=Test", "-c", "user.email=test@example.com"];
        all.extend_from_slice(args);
        run_git(dir, &all).unwrap()
    }

    fn file(path: &str, kind: ChangeKind) -> FileChange {
        FileChange {
            path: path.to_string(),
            orig_path: None,
            kind,
        }
    }

    fn paths(list: &[&str]) -> Vec<String> {
        list.iter().map(|p| p.to_string()).collect()
    }

    fn repo_with_commit() -> TempDir {
        let dir = TempDir::new().unwrap();
        git(dir.path(), &["init", "--quiet"]);
        fs::write(dir.path().join("edit.txt"), "one\n").unwrap();
        fs::write(dir.path().join("remove.txt"), "bye\n").unwrap();
        fs::write(dir.path().join("old.txt"), "rename me\n").unwrap();
        git(dir.path(), &["add", "."]);
        git(dir.path(), &["commit", "--quiet", "-m", "init"]);
        dir
    }

    #[test]
    fn stages_new_modified_and_deleted_files() {
        let dir = repo_with_commit();
        let root = dir.path();
        fs::write(root.join("edit.txt"), "two\n").unwrap();
        fs::remove_file(root.join("remove.txt")).unwrap();
        fs::write(root.join("new file.txt"), "new\n").unwrap();

        stage(root, &paths(&["edit.txt", "remove.txt", "new file.txt"])).unwrap();

        let status = read_status(root).unwrap();
        assert_eq!(
            status.staged,
            vec![
                file("edit.txt", ChangeKind::Modified),
                file("new file.txt", ChangeKind::Added),
                file("remove.txt", ChangeKind::Deleted),
            ]
        );
        assert!(status.unstaged.is_empty() && status.untracked.is_empty());
    }

    #[test]
    fn unstage_keeps_changes_in_working_tree() {
        let dir = repo_with_commit();
        let root = dir.path();
        fs::write(root.join("edit.txt"), "two\n").unwrap();
        fs::write(root.join("new.txt"), "new\n").unwrap();
        stage(root, &paths(&["edit.txt", "new.txt"])).unwrap();

        unstage(root, &paths(&["edit.txt", "new.txt"])).unwrap();

        let status = read_status(root).unwrap();
        assert!(status.staged.is_empty());
        assert_eq!(
            status.unstaged,
            vec![file("edit.txt", ChangeKind::Modified)]
        );
        assert_eq!(
            status.untracked,
            vec![file("new.txt", ChangeKind::Untracked)]
        );
        assert_eq!(fs::read_to_string(root.join("edit.txt")).unwrap(), "two\n");
    }

    #[test]
    fn unstage_rename_needs_both_paths() {
        let dir = repo_with_commit();
        let root = dir.path();
        git(root, &["mv", "old.txt", "renamed.txt"]);

        unstage(root, &paths(&["renamed.txt", "old.txt"])).unwrap();

        let status = read_status(root).unwrap();
        assert!(status.staged.is_empty());
        assert_eq!(status.unstaged, vec![file("old.txt", ChangeKind::Deleted)]);
        assert_eq!(
            status.untracked,
            vec![file("renamed.txt", ChangeKind::Untracked)]
        );
    }

    #[test]
    fn unstage_works_before_first_commit() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        git(root, &["init", "--quiet"]);
        fs::write(root.join("a.txt"), "a\n").unwrap();
        stage(root, &paths(&["a.txt"])).unwrap();

        unstage(root, &paths(&["a.txt"])).unwrap();

        let status = read_status(root).unwrap();
        assert!(status.staged.is_empty());
        assert_eq!(status.untracked, vec![file("a.txt", ChangeKind::Untracked)]);
        assert!(root.join("a.txt").is_file());
    }

    // '*' is not allowed in Windows file names, so this test is unix-only.
    #[cfg(unix)]
    #[test]
    fn treats_paths_literally_not_as_patterns() {
        let dir = repo_with_commit();
        let root = dir.path();
        fs::write(root.join("a*b.txt"), "star\n").unwrap();
        fs::write(root.join("aXb.txt"), "x\n").unwrap();

        stage(root, &paths(&["a*b.txt"])).unwrap();

        let status = read_status(root).unwrap();
        assert_eq!(status.staged, vec![file("a*b.txt", ChangeKind::Added)]);
        assert_eq!(
            status.untracked,
            vec![file("aXb.txt", ChangeKind::Untracked)]
        );
    }

    #[test]
    fn stages_more_files_than_one_batch() {
        let dir = repo_with_commit();
        let root = dir.path();
        let names: Vec<String> = (0..250).map(|i| format!("f{i}.txt")).collect();
        for name in &names {
            fs::write(root.join(name), "x\n").unwrap();
        }

        stage(root, &names).unwrap();

        let status = read_status(root).unwrap();
        assert_eq!(status.staged.len(), 250);
        assert_eq!(
            status,
            RepoStatus {
                staged: status.staged.clone(),
                ..RepoStatus::default()
            }
        );
    }

    #[test]
    fn reports_unknown_path_and_empty_selection() {
        let dir = repo_with_commit();
        let err = stage(dir.path(), &paths(&["does-not-exist.txt"])).unwrap_err();
        assert_eq!(err.kind, ErrorKind::GitFailed);
        assert_eq!(
            stage(dir.path(), &[]).unwrap_err().kind,
            ErrorKind::InvalidInput
        );
    }
}
