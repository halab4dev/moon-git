// Resolving conflicted files (step 24a), during a merge, rebase, cherry-pick
// or stash apply:
// - take ours / take theirs: the whole file from one side;
// - mark as resolved: the user fixed the file by hand; stage it as it is.
// All three end with the file staged, which is what tells git "resolved".
//
// While a file conflicts, the index holds up to three versions of it
// ("stages"): 1 = the common ancestor, 2 = ours, 3 = theirs. A side that
// deleted the file has no stage.

use std::fs;
use std::path::Path;

use crate::error::{AppError, ErrorKind};
use crate::git::diff::{Version, is_binary, read_blob, read_working_file};
use crate::git::runner::run_git;
use crate::git::status::read_status;
use crate::models::{ConflictChoice, ConflictVersions};

/// Resolves the conflicted `paths` with `choice`. `force`: mark as resolved
/// even if a file still contains conflict markers (the UI asks first).
pub fn resolve(
    repo: &Path,
    paths: &[String],
    choice: ConflictChoice,
    force: bool,
) -> Result<(), AppError> {
    if paths.is_empty() {
        return Err(invalid("No files selected.".to_string()));
    }
    // Only files that conflict now: the frontend can't make us touch others.
    let status = read_status(repo)?;
    for path in paths {
        if !status.conflicted.iter().any(|f| &f.path == path) {
            return Err(AppError::new(
                ErrorKind::NotFound,
                format!("\"{path}\" has no conflict (any more). The list has been reloaded."),
            ));
        }
    }
    if choice == ConflictChoice::Resolved && !force {
        let marked: Vec<&str> = paths
            .iter()
            .filter(|p| has_conflict_markers(&repo.join(p)))
            .map(|p| p.as_str())
            .collect();
        if !marked.is_empty() {
            return Err(AppError::new(
                ErrorKind::ConflictMarkers,
                format!(
                    "These files still contain conflict markers (<<<<<<<, =======, >>>>>>>): {}.",
                    marked.join(", ")
                ),
            ));
        }
    }

    for path in paths {
        match choice {
            ConflictChoice::Ours => take_side(repo, path, "--ours", 2)?,
            ConflictChoice::Theirs => take_side(repo, path, "--theirs", 3)?,
            ConflictChoice::Resolved => stage(repo, path)?,
        }
    }
    Ok(())
}

/// The texts the conflict solver window shows (step 24b): the common
/// ancestor, ours, theirs (stages 1, 2, 3 of the index), and the file as it
/// is now, with git's conflict markers.
pub fn versions(repo: &Path, path: &str) -> Result<ConflictVersions, AppError> {
    check_conflicted(repo, path)?;
    // ":1:path" = stage 1 of the index, and so on.
    let base = read_blob(repo, &format!(":1:{path}"))?;
    let ours = read_blob(repo, &format!(":2:{path}"))?;
    let theirs = read_blob(repo, &format!(":3:{path}"))?;
    let merged = read_working_file(repo, path)?;

    let all = [&base, &ours, &theirs, &merged];
    let too_large = all.iter().any(|v| matches!(v, Version::TooLarge));
    // Not editable as text: binary, or not UTF-8 (saving would change the bytes).
    let not_text = all.iter().any(|v| is_binary(v) || !is_utf8(v));
    let missing_side = matches!(ours, Version::Missing) || matches!(theirs, Version::Missing);
    let usable = !too_large && !not_text;
    Ok(ConflictVersions {
        base: text(base, usable),
        ours: text(ours, usable),
        theirs: text(theirs, usable),
        merged: text(merged, usable),
        binary: not_text,
        too_large,
        missing_side,
    })
}

/// Writes the edited result into the file. With `mark_resolved`, also
/// stages it (fails with ConflictMarkers if markers are left, unless `force`).
pub fn save(
    repo: &Path,
    path: &str,
    content: &str,
    mark_resolved: bool,
    force: bool,
) -> Result<(), AppError> {
    check_conflicted(repo, path)?;
    let file = repo.join(path);
    fs::write(&file, content).map_err(|err| {
        AppError::new(
            ErrorKind::Io,
            format!("Could not write {}: {err}", file.display()),
        )
    })?;
    if mark_resolved {
        resolve(repo, &[path.to_string()], ConflictChoice::Resolved, force)?;
    }
    Ok(())
}

fn check_conflicted(repo: &Path, path: &str) -> Result<(), AppError> {
    if read_status(repo)?.conflicted.iter().any(|f| f.path == path) {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::NotFound,
            format!("\"{path}\" has no conflict (any more)."),
        ))
    }
}

fn is_utf8(version: &Version) -> bool {
    match version {
        Version::Content(bytes) => std::str::from_utf8(bytes).is_ok(),
        _ => true,
    }
}

fn text(version: Version, usable: bool) -> Option<String> {
    match version {
        Version::Content(bytes) if usable => String::from_utf8(bytes).ok(),
        _ => None,
    }
}

// Puts one side's version in place and stages it. If that side deleted the
// file, taking it means deleting the file.
fn take_side(repo: &Path, path: &str, side: &str, stage_number: u8) -> Result<(), AppError> {
    if has_stage(repo, path, stage_number)? {
        run_git(repo, &["--literal-pathspecs", "checkout", side, "--", path])?;
        stage(repo, path)
    } else {
        // --ignore-unmatch: the file may already be gone from the disk.
        run_git(
            repo,
            &[
                "--literal-pathspecs",
                "rm",
                "--quiet",
                "--ignore-unmatch",
                "--",
                path,
            ],
        )?;
        Ok(())
    }
}

// `git ls-files --stage` lists "<mode> <id> <stage>\t<path>" for every
// version of the file in the index.
fn has_stage(repo: &Path, path: &str, stage_number: u8) -> Result<bool, AppError> {
    let output = run_git(
        repo,
        &[
            "--literal-pathspecs",
            "ls-files",
            "--stage",
            "-z",
            "--",
            path,
        ],
    )?;
    let wanted = stage_number.to_string();
    Ok(output
        .split('\0')
        .filter_map(|record| record.split_once('\t'))
        .any(|(info, _)| info.split(' ').nth(2) == Some(wanted.as_str())))
}

// --all: if the file was deleted in the working tree, stage the deletion.
fn stage(repo: &Path, path: &str) -> Result<(), AppError> {
    run_git(repo, &["--literal-pathspecs", "add", "--all", "--", path])?;
    Ok(())
}

// git writes these at the start of a line around each conflicting part.
// A file that is not text (or no longer exists) has none.
fn has_conflict_markers(file: &Path) -> bool {
    let Ok(bytes) = fs::read(file) else {
        return false;
    };
    let text = String::from_utf8_lossy(&bytes);
    let mut start = false;
    let mut middle = false;
    let mut end = false;
    for line in text.lines() {
        start |= line.starts_with("<<<<<<< ");
        middle |= line == "=======";
        end |= line.starts_with(">>>>>>> ");
    }
    start && middle && end
}

fn invalid(message: String) -> AppError {
    AppError::new(ErrorKind::InvalidInput, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_support::{commit_file, git};
    use tempfile::TempDir;

    // main and feature both change a.txt (content conflict); feature deletes
    // d.txt that main changes (delete/modify conflict). Merging feature into
    // main stops with both conflicted.
    fn merge_with_conflicts() -> TempDir {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        git(root, &["init", "--quiet", "--initial-branch=main"]);
        git(root, &["config", "user.name", "Test"]);
        git(root, &["config", "user.email", "test@example.com"]);
        commit_file(root, "a.txt");
        commit_file(root, "d.txt");
        git(root, &["switch", "--quiet", "--create", "feature"]);
        fs::write(root.join("a.txt"), "feature\n").unwrap();
        git(root, &["rm", "--quiet", "--", "d.txt"]);
        git(root, &["commit", "--quiet", "-am", "feature side"]);
        git(root, &["switch", "--quiet", "main"]);
        fs::write(root.join("a.txt"), "main\n").unwrap();
        fs::write(root.join("d.txt"), "main changed d\n").unwrap();
        git(root, &["commit", "--quiet", "-am", "main side"]);
        let _ = run_git(root, &["merge", "--no-edit", "feature"]);
        assert_eq!(read_status(root).unwrap().conflicted.len(), 2);
        dir
    }

    fn run(
        root: &Path,
        paths: &[&str],
        choice: ConflictChoice,
        force: bool,
    ) -> Result<(), AppError> {
        let paths: Vec<String> = paths.iter().map(|p| p.to_string()).collect();
        resolve(root, &paths, choice, force)
    }

    fn read(root: &Path, name: &str) -> String {
        fs::read_to_string(root.join(name)).unwrap()
    }

    #[test]
    fn take_ours_and_theirs() {
        let dir = merge_with_conflicts();
        let root = dir.path();
        run(root, &["a.txt"], ConflictChoice::Theirs, false).unwrap();
        assert_eq!(read(root, "a.txt"), "feature\n");

        // Ours kept d.txt (changed), theirs deleted it: ours keeps the change.
        run(root, &["d.txt"], ConflictChoice::Ours, false).unwrap();
        assert_eq!(read(root, "d.txt"), "main changed d\n");

        let status = read_status(root).unwrap();
        assert!(status.conflicted.is_empty());
        assert_eq!(status.staged.len(), 1); // a.txt; d.txt is as in HEAD
    }

    #[test]
    fn taking_the_side_that_deleted_the_file_deletes_it() {
        let dir = merge_with_conflicts();
        let root = dir.path();
        run(root, &["d.txt"], ConflictChoice::Theirs, false).unwrap();
        assert!(!root.join("d.txt").exists());
        let status = read_status(root).unwrap();
        assert!(status.staged.iter().any(|f| f.path == "d.txt"));
    }

    #[test]
    fn mark_as_resolved_checks_for_markers() {
        let dir = merge_with_conflicts();
        let root = dir.path();
        // Still as git left it, with markers.
        let err = run(root, &["a.txt"], ConflictChoice::Resolved, false).unwrap_err();
        assert_eq!(err.kind, ErrorKind::ConflictMarkers);
        assert!(err.message.contains("a.txt"));

        // Fixed by hand.
        fs::write(root.join("a.txt"), "main and feature\n").unwrap();
        run(root, &["a.txt"], ConflictChoice::Resolved, false).unwrap();
        assert_eq!(read(root, "a.txt"), "main and feature\n");

        // With force, even markers are accepted (the user confirmed).
        run(root, &["d.txt"], ConflictChoice::Resolved, true).unwrap();
        assert!(read_status(root).unwrap().conflicted.is_empty());

        // And the merge can be committed now.
        git(root, &["commit", "--quiet", "--no-edit"]);
        let parents = git(root, &["log", "-1", "--format=%P"]);
        assert_eq!(parents.split_whitespace().count(), 2);
    }

    #[test]
    fn only_conflicted_paths() {
        let dir = merge_with_conflicts();
        let root = dir.path();
        for bad in ["init.txt", "../x", "*.txt", "d.txt.missing"] {
            let err = run(root, &[bad], ConflictChoice::Ours, false).unwrap_err();
            assert_eq!(err.kind, ErrorKind::NotFound, "{bad}");
        }
        assert_eq!(read_status(root).unwrap().conflicted.len(), 2);
    }

    #[test]
    fn finds_markers_only_when_all_three_are_there() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("f.txt");
        fs::write(&file, "<<<<<<< HEAD\na\n=======\nb\n>>>>>>> feature\n").unwrap();
        assert!(has_conflict_markers(&file));
        // A line of "=" in a Markdown heading is not a conflict.
        fs::write(&file, "Title\n=======\ntext\n").unwrap();
        assert!(!has_conflict_markers(&file));
        assert!(!has_conflict_markers(&dir.path().join("missing")));
    }

    #[test]
    fn reads_the_three_versions_and_the_file_with_markers() {
        let dir = merge_with_conflicts();
        let root = dir.path();
        let v = versions(root, "a.txt").unwrap();
        assert_eq!(v.base.as_deref(), Some("a.txt\n"));
        assert_eq!(v.ours.as_deref(), Some("main\n"));
        assert_eq!(v.theirs.as_deref(), Some("feature\n"));
        assert!(
            v.merged
                .unwrap()
                .contains("<<<<<<< HEAD\nmain\n=======\nfeature\n>>>>>>> feature")
        );
        assert!(!v.binary && !v.too_large && !v.missing_side);

        // d.txt: deleted on their side.
        let v = versions(root, "d.txt").unwrap();
        assert!(v.missing_side);
        assert_eq!(v.theirs, None);

        assert_eq!(
            versions(root, "nope.txt").unwrap_err().kind,
            ErrorKind::NotFound
        );
    }

    #[test]
    fn saves_the_result_and_marks_it_resolved() {
        let dir = merge_with_conflicts();
        let root = dir.path();
        // Saving with markers left is fine; marking resolved is not (without force).
        save(
            root,
            "a.txt",
            "<<<<<<< x\n=======\n>>>>>>> y\n",
            false,
            false,
        )
        .unwrap();
        let err = save(
            root,
            "a.txt",
            "<<<<<<< x\n=======\n>>>>>>> y\n",
            true,
            false,
        )
        .unwrap_err();
        assert_eq!(err.kind, ErrorKind::ConflictMarkers);

        save(root, "a.txt", "main and feature\n", true, false).unwrap();

        assert_eq!(read(root, "a.txt"), "main and feature\n");
        let status = read_status(root).unwrap();
        assert_eq!(status.conflicted.len(), 1); // d.txt
        // Only conflicted files can be written this way.
        assert_eq!(
            save(root, "a.txt", "x", false, false).unwrap_err().kind,
            ErrorKind::NotFound
        );
    }
}
