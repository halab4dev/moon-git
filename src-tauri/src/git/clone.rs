use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AppError, ErrorKind};
use crate::git::operations::{Operations, run_git_with_progress};

/// Clones `url` into `parent_dir/folder_name` and returns the new folder.
///
/// If the clone fails or is cancelled and this call created the folder, the
/// half-finished folder is removed so a retry starts clean. A folder that
/// existed before (an empty one) is never removed.
pub fn clone_repo(
    ops: &Operations,
    op_id: &str,
    url: &str,
    parent_dir: &Path,
    folder_name: &str,
    on_line: impl FnMut(&str),
) -> Result<PathBuf, AppError> {
    let url = url.trim();
    if url.is_empty() {
        return Err(invalid("Enter the URL of the repository to clone."));
    }
    let folder_name = check_folder_name(folder_name)?;
    if !parent_dir.is_dir() {
        return Err(AppError::new(
            ErrorKind::PathNotFound,
            format!("Folder does not exist: {}", parent_dir.display()),
        ));
    }

    let target = parent_dir.join(folder_name);
    let existed_before = target.exists();
    if existed_before && !is_empty_dir(&target) {
        return Err(invalid(&format!(
            "{} already exists and is not an empty folder.",
            target.display()
        )));
    }

    // "--" ends the options: a URL or name starting with "-" can't be read as a flag.
    let args = ["clone", "--progress", "--", url, folder_name];
    let result = run_git_with_progress(ops, op_id, parent_dir, &args, on_line);

    if result.is_err() && !existed_before && target.exists() {
        remove_unfinished_clone(&target);
    }
    result.map(|()| target)
}

// Only called for a folder this clone created itself (it did not exist before).
fn remove_unfinished_clone(target: &Path) {
    if let Err(err) = fs::remove_dir_all(target) {
        eprintln!(
            "Could not remove unfinished clone {}: {err}",
            target.display()
        );
    }
}

/// Suggests a folder name from a URL, like git does:
/// "https://host/team/app.git" and "git@host:team/app.git" both give "app".
pub fn folder_name_from_url(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    let last = trimmed.rsplit(['/', ':', '\\']).next().unwrap_or("");
    last.strip_suffix(".git").unwrap_or(last).to_string()
}

fn check_folder_name(name: &str) -> Result<&str, AppError> {
    let name = name.trim();
    if name.is_empty() || name == "." || name == ".." {
        return Err(invalid("Enter a name for the new folder."));
    }
    if name.contains(['/', '\\']) {
        return Err(invalid("The folder name cannot contain / or \\."));
    }
    Ok(name)
}

fn is_empty_dir(path: &Path) -> bool {
    match fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_none(),
        Err(_) => false, // a file, or unreadable: treat as "not empty"
    }
}

fn invalid(message: &str) -> AppError {
    AppError::new(ErrorKind::InvalidInput, message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::runner::run_git;
    use tempfile::TempDir;

    // A local repository with one commit, to clone from without network.
    fn source_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        run_git(dir.path(), &["init", "--quiet"]).unwrap();
        fs::write(dir.path().join("README.md"), "hello\n").unwrap();
        run_git(dir.path(), &["add", "README.md"]).unwrap();
        run_git(
            dir.path(),
            &[
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.com",
                "commit",
                "--quiet",
                "-m",
                "init",
            ],
        )
        .unwrap();
        dir
    }

    #[test]
    fn clones_local_repository_and_reports_progress() {
        let source = source_repo();
        let dest = TempDir::new().unwrap();
        let mut lines = Vec::new();

        let target = clone_repo(
            &Operations::new(),
            "op",
            source.path().to_str().unwrap(),
            dest.path(),
            "copy with spaces",
            |line| lines.push(line.to_string()),
        )
        .unwrap();

        assert_eq!(target, dest.path().join("copy with spaces"));
        assert!(target.join("README.md").is_file());
        assert!(
            lines.iter().any(|l| l.starts_with("Cloning into")),
            "lines: {lines:?}"
        );
    }

    #[test]
    fn failed_clone_removes_the_new_folder() {
        let dest = TempDir::new().unwrap();
        let err = clone_repo(
            &Operations::new(),
            "op",
            "/no/such/repo",
            dest.path(),
            "x",
            |_| {},
        )
        .unwrap_err();

        assert_eq!(err.kind, ErrorKind::GitFailed);
        assert!(!dest.path().join("x").exists());
    }

    #[test]
    fn refuses_non_empty_target() {
        let source = source_repo();
        let dest = TempDir::new().unwrap();
        fs::create_dir(dest.path().join("taken")).unwrap();
        fs::write(dest.path().join("taken").join("file.txt"), "keep me").unwrap();

        let err = clone_repo(
            &Operations::new(),
            "op",
            source.path().to_str().unwrap(),
            dest.path(),
            "taken",
            |_| {},
        )
        .unwrap_err();

        assert_eq!(err.kind, ErrorKind::InvalidInput);
        assert!(dest.path().join("taken").join("file.txt").is_file());
    }

    #[test]
    fn keeps_existing_empty_folder_when_clone_fails() {
        let dest = TempDir::new().unwrap();
        fs::create_dir(dest.path().join("empty")).unwrap();

        clone_repo(
            &Operations::new(),
            "op",
            "/no/such/repo",
            dest.path(),
            "empty",
            |_| {},
        )
        .unwrap_err();

        assert!(dest.path().join("empty").is_dir());
    }

    #[test]
    fn validates_input() {
        let dest = TempDir::new().unwrap();
        let ops = Operations::new();
        assert_eq!(
            clone_repo(&ops, "op", "  ", dest.path(), "x", |_| {})
                .unwrap_err()
                .kind,
            ErrorKind::InvalidInput
        );
        assert_eq!(
            clone_repo(&ops, "op", "u", dest.path(), "a/b", |_| {})
                .unwrap_err()
                .kind,
            ErrorKind::InvalidInput
        );
        assert_eq!(
            clone_repo(&ops, "op", "u", dest.path(), "..", |_| {})
                .unwrap_err()
                .kind,
            ErrorKind::InvalidInput
        );
        assert_eq!(
            clone_repo(&ops, "op", "u", &dest.path().join("missing"), "x", |_| {})
                .unwrap_err()
                .kind,
            ErrorKind::PathNotFound
        );
    }

    #[test]
    fn guesses_folder_name_from_url() {
        assert_eq!(
            folder_name_from_url("https://github.com/team/app.git"),
            "app"
        );
        assert_eq!(folder_name_from_url("https://github.com/team/app/"), "app");
        assert_eq!(folder_name_from_url("git@github.com:team/app.git"), "app");
        assert_eq!(folder_name_from_url("ssh://git@host:2222/app"), "app");
        assert_eq!(folder_name_from_url(""), "");
    }
}
