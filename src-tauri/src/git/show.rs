// Details of one commit for the History tab (step 18): full message, author
// and committer, and the files it changed.

use std::path::Path;

use crate::error::{AppError, ErrorKind};
use crate::git::runner::run_git;
use crate::models::{ChangeKind, CommitDetails, FileChange};

// Author and committer can differ (e.g. after a rebase or cherry-pick).
// The full message (%B) is last, like the subject in log.rs.
const FORMAT: &str = "--format=%H%x1f%P%x1f%aN%x1f%aE%x1f%at%x1f%cN%x1f%cE%x1f%ct%x1f%B";
const FIELDS: usize = 9;

pub fn commit_details(repo: &Path, commit: &str) -> Result<CommitDetails, AppError> {
    check_commit_id(commit)?;
    // ^{commit}: the id must name a commit. Without it, the id of a file
    // (blob) would make `git show` print that file instead.
    let revision = format!("{commit}^{{commit}}");
    let output = run_git(
        repo,
        &[
            "show",
            "--no-patch",
            "--no-show-signature",
            "--no-color",
            FORMAT,
            &revision,
        ],
    )
    .map_err(|err| not_found(commit, err))?;

    let fields: Vec<&str> = output.splitn(FIELDS, '\x1f').collect();
    let [
        id,
        parents,
        author_name,
        author_email,
        author_time,
        committer_name,
        committer_email,
        committer_time,
        message,
    ] = fields[..]
    else {
        return Err(unexpected(&output));
    };
    let parents: Vec<String> = parents.split_whitespace().map(|p| p.to_string()).collect();
    let files = changed_files(repo, id, parents.first().map(|p| p.as_str()))?;

    Ok(CommitDetails {
        id: id.to_string(),
        author_name: author_name.to_string(),
        author_email: author_email.to_string(),
        author_time: author_time.parse().map_err(|_| unexpected(&output))?,
        committer_name: committer_name.to_string(),
        committer_email: committer_email.to_string(),
        committer_time: committer_time.parse().map_err(|_| unexpected(&output))?,
        message: message.trim_end().to_string(),
        parents,
        files,
    })
}

/// Files changed by `commit`, compared with its first parent (for a merge:
/// what the merge brought into the branch). A first commit is compared with
/// nothing, so every file is "added".
fn changed_files(
    repo: &Path,
    commit: &str,
    parent: Option<&str>,
) -> Result<Vec<FileChange>, AppError> {
    // diff-tree: the plumbing command, not affected by diff settings in the
    // user's config. -r: into subfolders. -M: detect renames. -z: NUL-separated.
    let mut args = vec![
        "diff-tree",
        "-r",
        "-z",
        "-M",
        "--name-status",
        "--no-commit-id",
    ];
    match parent {
        Some(parent) => args.extend([parent, commit]),
        None => args.extend(["--root", commit]),
    }
    parse_name_status(&run_git(repo, &args)?)
}

/// Reads "M\0path\0" and "R100\0old\0new\0" records.
pub fn parse_name_status(output: &str) -> Result<Vec<FileChange>, AppError> {
    let mut files = Vec::new();
    let mut tokens = output.split('\0').filter(|t| !t.is_empty());
    while let Some(status) = tokens.next() {
        // R and C carry a similarity score after the letter ("R100").
        let kind = match status.chars().next() {
            Some('A') => ChangeKind::Added,
            Some('M') => ChangeKind::Modified,
            Some('D') => ChangeKind::Deleted,
            Some('R') => ChangeKind::Renamed,
            Some('C') => ChangeKind::Copied,
            Some('T') => ChangeKind::TypeChanged,
            _ => return Err(unexpected(output)),
        };
        let first = tokens.next().ok_or_else(|| unexpected(output))?;
        let (path, orig_path) = match kind {
            ChangeKind::Renamed | ChangeKind::Copied => {
                let new_path = tokens.next().ok_or_else(|| unexpected(output))?;
                (new_path, Some(first.to_string()))
            }
            _ => (first, None),
        };
        files.push(FileChange {
            path: path.to_string(),
            orig_path,
            kind,
        });
    }
    Ok(files)
}

/// Accepts commit ids only (4 to 64 hex digits). Anything else, like a
/// branch name or text starting with "-", never reaches the command line.
pub fn check_commit_id(id: &str) -> Result<(), AppError> {
    let valid = (4..=64).contains(&id.len()) && id.chars().all(|c| c.is_ascii_hexdigit());
    if !valid {
        return Err(AppError::new(
            ErrorKind::InvalidInput,
            format!("Not a commit id: {id:?}"),
        ));
    }
    Ok(())
}

fn not_found(commit: &str, err: AppError) -> AppError {
    if err.kind == ErrorKind::GitFailed {
        return AppError::new(
            ErrorKind::NotFound,
            format!(
                "Commit {commit} was not found (it may have been removed by a rebase or amend)."
            ),
        );
    }
    err
}

fn unexpected(output: &str) -> AppError {
    AppError::new(
        ErrorKind::UnexpectedOutput,
        format!("Could not read commit details: {output:?}"),
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

    fn head(dir: &Path) -> String {
        git(dir, &["rev-parse", "HEAD"]).trim().to_string()
    }

    fn change(path: &str, orig: Option<&str>, kind: ChangeKind) -> FileChange {
        FileChange {
            path: path.to_string(),
            orig_path: orig.map(|o| o.to_string()),
            kind,
        }
    }

    #[test]
    fn parses_name_status_records() {
        let out = "M\0src/a b.rs\0R087\0old.txt\0new.txt\0A\0n\0D\0gone\0";
        assert_eq!(
            parse_name_status(out).unwrap(),
            vec![
                change("src/a b.rs", None, ChangeKind::Modified),
                change("new.txt", Some("old.txt"), ChangeKind::Renamed),
                change("n", None, ChangeKind::Added),
                change("gone", None, ChangeKind::Deleted),
            ]
        );
        assert!(parse_name_status("X\0a\0").is_err());
        assert!(parse_name_status("R100\0only-one\0").is_err());
    }

    #[test]
    fn reads_message_people_and_files() {
        let dir = new_repo();
        let root = dir.path();
        commit_file(root, "keep.txt");
        fs::write(root.join("old.txt"), "same content for rename\n").unwrap();
        git(root, &["add", "old.txt"]);
        git(root, &["commit", "--quiet", "-m", "base"]);
        let parent = head(root);

        fs::write(root.join("keep.txt"), "changed\n").unwrap();
        git(root, &["mv", "old.txt", "new.txt"]);
        fs::write(root.join("added.txt"), "a\n").unwrap();
        git(root, &["add", "-A"]);
        // Committer differs from author, like after a cherry-pick.
        git(
            root,
            &[
                "-c",
                "user.name=Committer",
                "-c",
                "user.email=c@example.com",
                "commit",
                "--quiet",
                "--author=Author <a@example.com>",
                "-m",
                "Subject line\n\nBody with details.",
            ],
        );

        let details = commit_details(root, &head(root)).unwrap();
        assert_eq!(details.parents, vec![parent]);
        assert_eq!(details.message, "Subject line\n\nBody with details.");
        assert_eq!(details.author_name, "Author");
        assert_eq!(details.author_email, "a@example.com");
        assert_eq!(details.committer_name, "Committer");
        assert!(details.author_time > 0 && details.committer_time > 0);
        assert_eq!(
            details.files,
            vec![
                change("added.txt", None, ChangeKind::Added),
                change("keep.txt", None, ChangeKind::Modified),
                change("new.txt", Some("old.txt"), ChangeKind::Renamed),
            ]
        );
    }

    #[test]
    fn first_commit_lists_all_files_as_added() {
        let dir = new_repo();
        fs::write(dir.path().join("a.txt"), "a\n").unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub/b.txt"), "b\n").unwrap();
        git(dir.path(), &["add", "."]);
        git(dir.path(), &["commit", "--quiet", "-m", "root"]);

        let details = commit_details(dir.path(), &head(dir.path())).unwrap();
        assert!(details.parents.is_empty());
        assert_eq!(
            details.files,
            vec![
                change("a.txt", None, ChangeKind::Added),
                change("sub/b.txt", None, ChangeKind::Added),
            ]
        );
    }

    #[test]
    fn merge_lists_changes_against_first_parent() {
        let dir = new_repo();
        let root = dir.path();
        commit_file(root, "base.txt");
        git(root, &["checkout", "--quiet", "-b", "side"]);
        commit_file(root, "from-side.txt");
        git(root, &["checkout", "--quiet", "main"]);
        commit_file(root, "on-main.txt");
        git(root, &["merge", "--quiet", "--no-ff", "--no-edit", "side"]);

        let details = commit_details(root, &head(root)).unwrap();
        assert_eq!(details.parents.len(), 2);
        // What the merge brought into main: the side branch's file.
        assert_eq!(
            details.files,
            vec![change("from-side.txt", None, ChangeKind::Added)]
        );
    }

    #[test]
    fn accepts_only_commit_ids() {
        let dir = new_repo();
        commit_file(dir.path(), "a.txt");
        for bad in ["main", "--all", "abc", "HEAD", "1234567890abcdefg"] {
            let err = commit_details(dir.path(), bad).unwrap_err();
            assert_eq!(err.kind, ErrorKind::InvalidInput, "{bad}");
        }
        let missing = commit_details(dir.path(), "0123456789abcdef0123").unwrap_err();
        assert_eq!(missing.kind, ErrorKind::NotFound);
        // A short id works like in the terminal.
        let short = &head(dir.path())[..7];
        assert_eq!(
            commit_details(dir.path(), short).unwrap().id,
            head(dir.path())
        );
    }
}
