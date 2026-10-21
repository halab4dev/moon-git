// Loads the two versions of one file for the side-by-side diff view.
// The frontend editor (Monaco) computes and draws the differences itself.
//
// Where each version comes from:
//   HEAD          `git cat-file blob HEAD:<path>`   last commit
//   Index         `git cat-file blob :0:<path>`     staging area
//   Working Tree  the file on disk
//   a commit      `git cat-file blob <id>:<path>`   History tab (read_commit_diff)

use std::fs;
use std::io;
use std::path::{Component, Path};

use crate::error::{AppError, ErrorKind};
use crate::git::runner::{run_git, run_git_bytes};
use crate::git::show::check_commit_id;
use crate::models::{DiffMode, FileDiff};

/// Bigger files are not sent to the editor: it would become slow.
const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;

/// git's own rule: a NUL byte in the first 8000 bytes means "binary".
const BINARY_CHECK_LEN: usize = 8000;

/// One version of the file.
enum Version {
    Missing,
    TooLarge,
    Content(Vec<u8>),
}

/// `orig_path` is the old name of a renamed file (only used for staged changes).
pub fn read_diff(
    repo: &Path,
    path: &str,
    orig_path: Option<&str>,
    mode: DiffMode,
) -> Result<FileDiff, AppError> {
    check_relative(path)?;
    if let Some(orig) = orig_path {
        check_relative(orig)?;
    }

    // ":0:" = stage 0 of the index. Written out so a file named like "1:x"
    // is not read as ":1:x" (stage 1, used during merges).
    let index_spec = format!(":0:{path}");
    let (old, new, old_label, new_label) = match mode {
        DiffMode::Unstaged => (
            read_blob(repo, &index_spec)?,
            read_working_file(repo, path)?,
            "Index",
            "Working Tree",
        ),
        DiffMode::Staged => {
            let head_spec = format!("HEAD:{}", orig_path.unwrap_or(path));
            (
                read_blob(repo, &head_spec)?,
                read_blob(repo, &index_spec)?,
                "HEAD",
                "Index",
            )
        }
        DiffMode::Conflict => (
            read_blob(repo, &format!("HEAD:{path}"))?,
            read_working_file(repo, path)?,
            "HEAD",
            "Working Tree",
        ),
    };
    Ok(make_diff(old, new, old_label, new_label))
}

/// One file of a commit in the History tab: its version in `parent` (the
/// first parent; None for the first commit) against its version in `commit`.
/// `orig_path`: the old name of a renamed file, read from the parent.
pub fn read_commit_diff(
    repo: &Path,
    commit: &str,
    parent: Option<&str>,
    path: &str,
    orig_path: Option<&str>,
) -> Result<FileDiff, AppError> {
    check_commit_id(commit)?;
    check_relative(path)?;
    if let Some(orig) = orig_path {
        check_relative(orig)?;
    }
    let old = match parent {
        Some(parent) => {
            check_commit_id(parent)?;
            read_blob(repo, &format!("{parent}:{}", orig_path.unwrap_or(path)))?
        }
        None => Version::Missing,
    };
    let new = read_blob(repo, &format!("{commit}:{path}"))?;
    let old_label = parent.map_or("(none)".to_string(), short_id);
    Ok(make_diff(old, new, &old_label, &short_id(commit)))
}

fn short_id(id: &str) -> String {
    id.chars().take(8).collect()
}

fn make_diff(old: Version, new: Version, old_label: &str, new_label: &str) -> FileDiff {
    let mut diff = FileDiff {
        old_label: old_label.to_string(),
        new_label: new_label.to_string(),
        old_text: None,
        new_text: None,
        binary: false,
        too_large: false,
    };

    if matches!(old, Version::TooLarge) || matches!(new, Version::TooLarge) {
        diff.too_large = true;
        return diff;
    }
    if is_binary(&old) || is_binary(&new) {
        diff.binary = true;
        return diff;
    }
    diff.old_text = to_text(old);
    diff.new_text = to_text(new);
    diff
}

fn is_binary(version: &Version) -> bool {
    match version {
        Version::Content(bytes) => {
            let checked = &bytes[..bytes.len().min(BINARY_CHECK_LEN)];
            checked.contains(&0)
        }
        _ => false,
    }
}

fn to_text(version: Version) -> Option<String> {
    match version {
        // Lossy: text in another encoding (e.g. Latin-1) shows "�" for some characters.
        Version::Content(bytes) => Some(String::from_utf8_lossy(&bytes).into_owned()),
        _ => None,
    }
}

/// Reads a file version stored in git, e.g. "HEAD:src/a.rs".
fn read_blob(repo: &Path, spec: &str) -> Result<Version, AppError> {
    // `cat-file -s` prints the size, and fails when that version does not exist
    // (new file, deleted file, no commit yet). That failure just means "Missing".
    let size = match run_git(repo, &["cat-file", "-s", spec]) {
        Ok(out) => out.trim().parse::<u64>().unwrap_or(0),
        Err(_) => return Ok(Version::Missing),
    };
    if size > MAX_FILE_SIZE {
        return Ok(Version::TooLarge);
    }
    Ok(Version::Content(run_git_bytes(
        repo,
        &["cat-file", "blob", spec],
    )?))
}

fn read_working_file(repo: &Path, path: &str) -> Result<Version, AppError> {
    let full = repo.join(path);
    // symlink_metadata looks at the link itself instead of following it.
    let meta = match fs::symlink_metadata(&full) {
        Ok(meta) => meta,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Version::Missing),
        Err(err) => return Err(read_error(&full, err)),
    };

    if meta.file_type().is_symlink() {
        // git stores a symlink as a tiny file holding the target path; show the same.
        let target = fs::read_link(&full).map_err(|err| read_error(&full, err))?;
        return Ok(Version::Content(
            target.to_string_lossy().into_owned().into_bytes(),
        ));
    }
    if !meta.is_file() {
        return Ok(Version::Missing); // e.g. a submodule folder
    }
    if meta.len() > MAX_FILE_SIZE {
        return Ok(Version::TooLarge);
    }
    let bytes = fs::read(&full).map_err(|err| read_error(&full, err))?;
    Ok(Version::Content(bytes))
}

/// Only plain relative paths inside the repository, e.g. "src/a.rs".
/// Rejects "/etc/passwd", "../other", "C:\x" so nothing outside the repo is read.
fn check_relative(path: &str) -> Result<(), AppError> {
    let inside = !path.is_empty()
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir));
    if !inside {
        return Err(AppError::new(
            ErrorKind::InvalidInput,
            format!("Not a path inside the repository: {path}"),
        ));
    }
    Ok(())
}

fn read_error(path: &Path, err: io::Error) -> AppError {
    AppError::new(
        ErrorKind::Io,
        format!("Could not read {}: {err}", path.display()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn git(dir: &Path, args: &[&str]) -> String {
        let mut all = vec!["-c", "user.name=Test", "-c", "user.email=test@example.com"];
        all.extend_from_slice(args);
        run_git(dir, &all).unwrap()
    }

    fn repo_with(files: &[(&str, &str)]) -> TempDir {
        let dir = TempDir::new().unwrap();
        git(dir.path(), &["init", "--quiet"]);
        for (name, content) in files {
            fs::write(dir.path().join(name), content).unwrap();
        }
        git(dir.path(), &["add", "."]);
        git(dir.path(), &["commit", "--quiet", "-m", "init"]);
        dir
    }

    fn texts(diff: &FileDiff) -> (Option<&str>, Option<&str>) {
        (diff.old_text.as_deref(), diff.new_text.as_deref())
    }

    #[test]
    fn unstaged_compares_index_with_working_tree() {
        let dir = repo_with(&[("a.txt", "one\n")]);
        fs::write(dir.path().join("a.txt"), "two\n").unwrap();

        let diff = read_diff(dir.path(), "a.txt", None, DiffMode::Unstaged).unwrap();

        assert_eq!(texts(&diff), (Some("one\n"), Some("two\n")));
        assert_eq!(
            (diff.old_label.as_str(), diff.new_label.as_str()),
            ("Index", "Working Tree")
        );
    }

    #[test]
    fn staged_and_unstaged_show_different_pairs() {
        let dir = repo_with(&[("a.txt", "head\n")]);
        fs::write(dir.path().join("a.txt"), "index\n").unwrap();
        git(dir.path(), &["add", "a.txt"]);
        fs::write(dir.path().join("a.txt"), "disk\n").unwrap();

        let staged = read_diff(dir.path(), "a.txt", None, DiffMode::Staged).unwrap();
        let unstaged = read_diff(dir.path(), "a.txt", None, DiffMode::Unstaged).unwrap();

        assert_eq!(texts(&staged), (Some("head\n"), Some("index\n")));
        assert_eq!(texts(&unstaged), (Some("index\n"), Some("disk\n")));
    }

    #[test]
    fn new_and_deleted_files_have_a_missing_side() {
        let dir = repo_with(&[("gone.txt", "bye\n")]);
        fs::write(dir.path().join("new.txt"), "hello\n").unwrap();
        fs::remove_file(dir.path().join("gone.txt")).unwrap();

        let untracked = read_diff(dir.path(), "new.txt", None, DiffMode::Unstaged).unwrap();
        let deleted = read_diff(dir.path(), "gone.txt", None, DiffMode::Unstaged).unwrap();
        git(dir.path(), &["add", "new.txt"]);
        let added = read_diff(dir.path(), "new.txt", None, DiffMode::Staged).unwrap();

        assert_eq!(texts(&untracked), (None, Some("hello\n")));
        assert_eq!(texts(&deleted), (Some("bye\n"), None));
        assert_eq!(texts(&added), (None, Some("hello\n")));
    }

    #[test]
    fn staged_rename_reads_old_name_from_head() {
        let dir = repo_with(&[("old.txt", "same\n")]);
        git(dir.path(), &["mv", "old.txt", "new.txt"]);

        let diff = read_diff(dir.path(), "new.txt", Some("old.txt"), DiffMode::Staged).unwrap();

        assert_eq!(texts(&diff), (Some("same\n"), Some("same\n")));
    }

    #[test]
    fn works_before_first_commit() {
        let dir = TempDir::new().unwrap();
        git(dir.path(), &["init", "--quiet"]);
        fs::write(dir.path().join("a.txt"), "a\n").unwrap();
        git(dir.path(), &["add", "a.txt"]);

        let diff = read_diff(dir.path(), "a.txt", None, DiffMode::Staged).unwrap();

        assert_eq!(texts(&diff), (None, Some("a\n")));
    }

    #[test]
    fn detects_binary_and_large_files() {
        let dir = repo_with(&[("img.bin", "x")]);
        fs::write(
            dir.path().join("img.bin"),
            [0x89, b'P', b'N', b'G', 0, 1, 2],
        )
        .unwrap();
        fs::write(
            dir.path().join("big.txt"),
            "x".repeat(MAX_FILE_SIZE as usize + 1),
        )
        .unwrap();

        let binary = read_diff(dir.path(), "img.bin", None, DiffMode::Unstaged).unwrap();
        let large = read_diff(dir.path(), "big.txt", None, DiffMode::Unstaged).unwrap();

        assert!(binary.binary && binary.old_text.is_none() && binary.new_text.is_none());
        assert!(large.too_large && large.new_text.is_none());
    }

    #[test]
    fn conflict_shows_markers_from_working_tree() {
        let dir = repo_with(&[("a.txt", "base\n")]);
        let root = dir.path();
        git(root, &["checkout", "--quiet", "-b", "other"]);
        fs::write(root.join("a.txt"), "other\n").unwrap();
        git(root, &["commit", "--quiet", "-am", "other"]);
        git(root, &["checkout", "--quiet", "-"]);
        fs::write(root.join("a.txt"), "main\n").unwrap();
        git(root, &["commit", "--quiet", "-am", "main"]);
        let mut args = vec!["-c", "user.name=Test", "-c", "user.email=test@example.com"];
        args.extend_from_slice(&["merge", "other"]);
        assert!(run_git(root, &args).is_err());

        let diff = read_diff(root, "a.txt", None, DiffMode::Conflict).unwrap();

        assert_eq!(diff.old_text.as_deref(), Some("main\n"));
        assert!(diff.new_text.unwrap().contains("<<<<<<<"));
    }

    // ':' is not allowed in Windows file names, so this test is unix-only.
    #[cfg(unix)]
    #[test]
    fn file_name_with_colon_is_read_from_index() {
        let dir = repo_with(&[("1:x", "head\n")]);
        fs::write(dir.path().join("1:x"), "index\n").unwrap();
        git(dir.path(), &["add", "1:x"]);

        let diff = read_diff(dir.path(), "1:x", None, DiffMode::Staged).unwrap();

        assert_eq!(texts(&diff), (Some("head\n"), Some("index\n")));
    }

    #[test]
    fn commit_diff_compares_with_the_parent() {
        let dir = repo_with(&[("a.txt", "one\n"), ("old.txt", "rename me\n")]);
        let root = dir.path();
        let parent = git(root, &["rev-parse", "HEAD"]).trim().to_string();
        fs::write(root.join("a.txt"), "two\n").unwrap();
        git(root, &["mv", "old.txt", "new.txt"]);
        fs::write(root.join("added.txt"), "fresh\n").unwrap();
        git(root, &["add", "-A"]);
        git(root, &["commit", "--quiet", "-m", "change"]);
        let commit = git(root, &["rev-parse", "HEAD"]).trim().to_string();

        let changed = read_commit_diff(root, &commit, Some(&parent), "a.txt", None).unwrap();
        assert_eq!(texts(&changed), (Some("one\n"), Some("two\n")));
        assert_eq!(changed.old_label, &parent[..8]);
        assert_eq!(changed.new_label, &commit[..8]);

        let renamed =
            read_commit_diff(root, &commit, Some(&parent), "new.txt", Some("old.txt")).unwrap();
        assert_eq!(texts(&renamed), (Some("rename me\n"), Some("rename me\n")));

        let added = read_commit_diff(root, &commit, Some(&parent), "added.txt", None).unwrap();
        assert_eq!(texts(&added), (None, Some("fresh\n")));

        // First commit: nothing to compare with.
        let first = read_commit_diff(root, &parent, None, "a.txt", None).unwrap();
        assert_eq!(texts(&first), (None, Some("one\n")));

        let err = read_commit_diff(root, "--all", None, "a.txt", None).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
    }

    #[test]
    fn rejects_paths_outside_the_repository() {
        let dir = repo_with(&[("a.txt", "a\n")]);
        for bad in ["", "../a.txt", "/etc/passwd", "src/../../x"] {
            let err = read_diff(dir.path(), bad, None, DiffMode::Unstaged).unwrap_err();
            assert_eq!(err.kind, ErrorKind::InvalidInput, "path: {bad:?}");
        }
        let err = read_diff(dir.path(), "a.txt", Some("../b"), DiffMode::Staged).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
    }
}
