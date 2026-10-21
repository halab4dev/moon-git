use std::path::Path;

use crate::error::{AppError, ErrorKind};
use crate::git::runner::run_git;
use crate::models::RepoInfo;

/// Checks that `path` is inside a git working tree and returns the repo root.
/// A subfolder of a repo is accepted and resolved to its root.
pub fn check_repo(path: &Path) -> Result<RepoInfo, AppError> {
    let stdout = match run_git(path, &["rev-parse", "--show-toplevel"]) {
        Ok(out) => out,
        Err(err) if err.kind == ErrorKind::GitFailed && err.message.contains("not a git repository") => {
            return Err(AppError::new(
                ErrorKind::NotARepository,
                format!("{} is not a git repository.", path.display()),
            ));
        }
        Err(err) => return Err(err),
    };

    // Only strip the trailing newline: a folder name may legally end with a space.
    let root = stdout.trim_end_matches(['\n', '\r']).to_string();
    let name = match Path::new(&root).file_name() {
        Some(n) => n.to_string_lossy().to_string(),
        None => root.clone(),
    };

    Ok(RepoInfo { root, name })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn init_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        run_git(dir.path(), &["init", "--quiet"]).unwrap();
        dir
    }

    // Temp folders can sit behind symlinks (macOS: /var -> /private/var) and git
    // prints the resolved path, so compare canonical forms.
    fn same_path(a: &Path, b: &Path) -> bool {
        fs::canonicalize(a).unwrap() == fs::canonicalize(b).unwrap()
    }

    #[test]
    fn returns_root_of_repo() {
        let dir = init_repo();
        let info = check_repo(dir.path()).unwrap();
        assert!(same_path(Path::new(&info.root), dir.path()), "root: {}", info.root);
        assert_eq!(info.name, dir.path().file_name().unwrap().to_string_lossy());
    }

    #[test]
    fn resolves_subfolder_to_root() {
        let dir = init_repo();
        let sub = dir.path().join("a folder with spaces");
        fs::create_dir(&sub).unwrap();
        let info = check_repo(&sub).unwrap();
        assert!(same_path(Path::new(&info.root), dir.path()), "root: {}", info.root);
    }

    #[test]
    fn rejects_plain_folder() {
        let dir = TempDir::new().unwrap();
        let err = check_repo(dir.path()).unwrap_err();
        assert_eq!(err.kind, ErrorKind::NotARepository);
    }

    #[test]
    fn rejects_missing_folder() {
        let dir = TempDir::new().unwrap();
        let err = check_repo(&dir.path().join("nope")).unwrap_err();
        assert_eq!(err.kind, ErrorKind::PathNotFound);
    }
}
