// Stashes (step 20): put local changes aside, list what was put aside.
//
// A stash is stored as a commit (with 2 or 3 parents) that only the reflog of
// refs/stash points at, so it never shows up in the history of a branch.

use std::path::Path;

use crate::error::{AppError, ErrorKind};
use crate::git::repo::has_commits;
use crate::git::runner::run_git;
use crate::git::status::read_status;
use crate::models::StashEntry;

// One record per stash (-z: NUL-terminated), fields split by \x1f.
// %ct: when the stash was made. %gs: the reflog message, e.g.
// "On main: my message" or "WIP on main: 1234abc last commit subject".
const FORMAT: &str = "--format=%H%x1f%P%x1f%ct%x1f%gs";

/// Stashes the staged and unstaged changes of tracked files, and with
/// `include_untracked` the untracked files too (never the ignored ones).
/// An empty `message` lets git write "WIP on <branch>: …".
pub fn create(repo: &Path, message: &str, include_untracked: bool) -> Result<(), AppError> {
    if !has_commits(repo) {
        return Err(invalid(
            "There is no commit yet. Make a first commit before stashing.",
        ));
    }
    // Checked here because `git stash` with nothing to stash prints a note
    // and exits with 0, so nothing would tell the user that nothing happened.
    let status = read_status(repo)?;
    if !status.conflicted.is_empty() {
        return Err(invalid(
            "Some files have conflicts. Resolve them before stashing.",
        ));
    }
    let tracked = !status.staged.is_empty() || !status.unstaged.is_empty();
    let untracked = !status.untracked.is_empty();
    if !tracked && !(include_untracked && untracked) {
        return Err(invalid(if untracked {
            "Only untracked files changed. Tick \"Include untracked files\" to stash them."
        } else {
            "There are no local changes to stash."
        }));
    }

    let mut args = vec![
        "stash".to_string(),
        "push".to_string(),
        "--quiet".to_string(),
    ];
    if include_untracked {
        args.push("--include-untracked".to_string());
    }
    let message = message.trim();
    if !message.is_empty() {
        // "--message=…" in one argument: the text can never be read as an option.
        args.push(format!("--message={message}"));
    }
    let args: Vec<&str> = args.iter().map(|a| a.as_str()).collect();
    run_git(repo, &args)?;
    Ok(())
}

/// All stashes, newest first (index 0 = `stash@{0}`).
pub fn list(repo: &Path) -> Result<Vec<StashEntry>, AppError> {
    // `stash list` (not `log -g refs/stash`): prints nothing, instead of
    // failing, when there is no stash yet.
    let output = run_git(
        repo,
        &[
            "stash",
            "list",
            "-z",
            "--no-show-signature",
            "--no-color",
            FORMAT,
        ],
    )?;
    parse_list(&output)
}

fn parse_list(output: &str) -> Result<Vec<StashEntry>, AppError> {
    output
        .split('\0')
        .filter(|record| !record.is_empty())
        .enumerate()
        .map(|(index, record)| parse_entry(index, record))
        .collect()
}

fn parse_entry(index: usize, record: &str) -> Result<StashEntry, AppError> {
    let fields: Vec<&str> = record.splitn(4, '\x1f').collect();
    let [id, parents, time, message] = fields[..] else {
        return Err(unexpected(record));
    };
    // Parents: the commit that was checked out, the staged changes, and
    // (only with untracked files) a third commit holding those files.
    let parents: Vec<&str> = parents.split_whitespace().collect();
    if parents.len() < 2 {
        return Err(unexpected(record));
    }
    Ok(StashEntry {
        index,
        id: id.to_string(),
        base: parents[0].to_string(),
        untracked: parents.get(2).map(|p| p.to_string()),
        time: time.parse().map_err(|_| unexpected(record))?,
        message: message.to_string(),
    })
}

fn invalid(message: &str) -> AppError {
    AppError::new(ErrorKind::InvalidInput, message.to_string())
}

fn unexpected(record: &str) -> AppError {
    AppError::new(
        ErrorKind::UnexpectedOutput,
        format!("Could not read this line of `git stash list`: {record:?}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_support::{commit_file, git};
    use std::fs;
    use tempfile::TempDir;

    fn new_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        git(dir.path(), &["init", "--quiet", "--initial-branch=main"]);
        git(dir.path(), &["config", "user.name", "Test"]);
        git(dir.path(), &["config", "user.email", "test@example.com"]);
        dir
    }

    #[test]
    fn stashes_tracked_changes_with_a_message() {
        let dir = new_repo();
        let root = dir.path();
        commit_file(root, "a.txt");
        fs::write(root.join("a.txt"), "changed\n").unwrap();
        fs::write(root.join("staged.txt"), "new\n").unwrap();
        git(root, &["add", "--", "staged.txt"]);
        fs::write(root.join("untracked.txt"), "u\n").unwrap();

        create(root, "  --my work  ", false).unwrap();

        let status = read_status(root).unwrap();
        assert!(status.staged.is_empty() && status.unstaged.is_empty());
        // Untracked files stay where they are without the option.
        assert_eq!(status.untracked.len(), 1);
        let stashes = list(root).unwrap();
        assert_eq!(stashes.len(), 1);
        assert_eq!(stashes[0].index, 0);
        assert_eq!(stashes[0].message, "On main: --my work");
        assert_eq!(stashes[0].untracked, None);
        let head = git(root, &["rev-parse", "HEAD"]);
        assert_eq!(stashes[0].base, head.trim());
    }

    #[test]
    fn includes_untracked_files_when_asked() {
        let dir = new_repo();
        let root = dir.path();
        commit_file(root, "a.txt");
        fs::write(root.join("untracked.txt"), "u\n").unwrap();

        create(root, "", true).unwrap();

        assert!(read_status(root).unwrap().untracked.is_empty());
        let stash = &list(root).unwrap()[0];
        assert!(stash.message.starts_with("WIP on main: "));
        // The third parent holds the untracked file.
        let untracked = stash.untracked.as_deref().unwrap();
        let files = git(root, &["ls-tree", "--name-only", untracked]);
        assert_eq!(files, "untracked.txt\n");
    }

    #[test]
    fn lists_newest_first() {
        let dir = new_repo();
        let root = dir.path();
        commit_file(root, "a.txt");
        for text in ["one", "two"] {
            fs::write(root.join("a.txt"), text).unwrap();
            create(root, text, false).unwrap();
        }
        let messages: Vec<String> = list(root).unwrap().into_iter().map(|s| s.message).collect();
        assert_eq!(messages, ["On main: two", "On main: one"]);
    }

    #[test]
    fn refuses_when_there_is_nothing_to_stash() {
        let dir = new_repo();
        let root = dir.path();
        commit_file(root, "a.txt");
        let err = create(root, "x", false).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);

        // Only an untracked file: a hint to tick the option.
        fs::write(root.join("untracked.txt"), "u\n").unwrap();
        let err = create(root, "x", false).unwrap_err();
        assert!(err.message.contains("Include untracked files"));
        assert!(list(root).unwrap().is_empty());
    }

    #[test]
    fn refuses_before_the_first_commit() {
        let dir = new_repo();
        fs::write(dir.path().join("a.txt"), "a\n").unwrap();
        let err = create(dir.path(), "", true).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
    }

    #[test]
    fn empty_list_without_stashes() {
        let dir = new_repo();
        commit_file(dir.path(), "a.txt");
        assert!(list(dir.path()).unwrap().is_empty());
    }

    #[test]
    fn parses_records() {
        let output = "aaa\x1fbbb ccc\x1f100\x1fOn main: x\x1fy\0ddd\x1feee fff ggg\x1f200\x1fWIP on dev: 1 z\0";
        let stashes = parse_list(output).unwrap();
        assert_eq!(stashes.len(), 2);
        // A \x1f inside the message stays part of it.
        assert_eq!(stashes[0].message, "On main: x\x1fy");
        assert_eq!(stashes[0].untracked, None);
        assert_eq!(stashes[1].index, 1);
        assert_eq!(stashes[1].base, "eee");
        assert_eq!(stashes[1].untracked.as_deref(), Some("ggg"));
        assert_eq!(stashes[1].time, 200);
        assert!(parse_list("only-id\0").is_err());
    }
}
