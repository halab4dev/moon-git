// Reads `git status --porcelain=v2 --branch -z`.
//
// Each record is separated by NUL ("\0"), so paths may contain spaces or
// newlines safely. Record types (first character):
//   1 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <path>                 changed file
//   2 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <Xscore> <path>\0<orig>  renamed/copied
//   u <XY> <sub> <m1> <m2> <m3> <mW> <h1> <h2> <h3> <path>         merge conflict
//   ? <path>                                                       untracked
//   # branch.<key> <value>                                       header (--branch)
// XY: X = change in the index (staged), Y = change in the working tree; "." = none.
// Docs: `git help status`, section "Porcelain Format Version 2".

use std::fs;
use std::path::Path;

use crate::error::{AppError, ErrorKind};
use crate::git::rebase::read_rebase_state;
use crate::git::runner::run_git;
use crate::models::{BranchInfo, ChangeKind, FileChange, MergeState, RepoStatus};

// Not part of `git status --porcelain`: git keeps the state of a merge or
// rebase that waits halfway in files inside .git:
// - MERGE_HEAD = the commit being merged, MERGE_MSG = its message (step 21b);
// - the folder rebase-merge (or rebase-apply) during a rebase (step 22b).
// --git-path: where those are, also for worktrees and custom git dirs.
// One git call for all four paths.
fn add_operation_state(repo: &Path, status: &mut RepoStatus) -> Result<(), AppError> {
    let output = run_git(
        repo,
        &[
            "rev-parse",
            "--git-path",
            "MERGE_HEAD",
            "--git-path",
            "MERGE_MSG",
            "--git-path",
            "rebase-merge",
            "--git-path",
            "rebase-apply",
        ],
    )?;
    let lines: Vec<&str> = output.lines().collect();
    let [head, msg, rebase_merge, rebase_apply] = lines[..] else {
        return Err(unexpected(&output));
    };
    // The paths are relative to the repo folder (or absolute; join keeps those).
    if repo.join(head).is_file() {
        let raw = fs::read_to_string(repo.join(msg)).unwrap_or_default();
        status.merge = Some(MergeState {
            message: clean_message(&raw),
        });
    }
    status.rebase = read_rebase_state(&repo.join(rebase_merge))
        .or_else(|| read_rebase_state(&repo.join(rebase_apply)));
    Ok(())
}

// git adds hints as "#" lines (e.g. "# Conflicts:" with the file names);
// an editor would drop them, so the prefilled message drops them too.
fn clean_message(raw: &str) -> String {
    raw.lines()
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

pub fn read_status(repo: &Path) -> Result<RepoStatus, AppError> {
    let output = run_git(
        repo,
        &[
            // status may refresh the index and take .git/index.lock to do it; a GUI
            // polling in the background must not block git commands run in a terminal.
            "--no-optional-locks",
            "status",
            "--porcelain=v2",
            // Also print the branch headers (name, upstream, ahead/behind).
            "--branch",
            "-z",
            // List every file inside new folders, not just "folder/".
            "--untracked-files=all",
        ],
    )?;
    let mut status = parse_status(&output)?;
    add_operation_state(repo, &mut status)?;
    Ok(status)
}

pub fn parse_status(output: &str) -> Result<RepoStatus, AppError> {
    let mut status = RepoStatus::default();
    let mut records = output.split('\0');
    let mut has_ahead_behind = false;

    while let Some(record) = records.next() {
        if record.is_empty() {
            continue; // the output ends with a NUL, leaving one empty piece
        }
        let (tag, rest) = record.split_once(' ').ok_or_else(|| unexpected(record))?;
        match tag {
            "1" => {
                let fields = split_fields(rest, 8, record)?;
                push_changes(&mut status, fields[0], fields[7], None, record)?;
            }
            "2" => {
                let fields = split_fields(rest, 9, record)?;
                // The original path is the next NUL-separated record.
                let orig_path = match records.next() {
                    Some(orig) if !orig.is_empty() => orig,
                    _ => return Err(unexpected(record)),
                };
                push_changes(&mut status, fields[0], fields[8], Some(orig_path), record)?;
            }
            "u" => {
                let fields = split_fields(rest, 10, record)?;
                status
                    .conflicted
                    .push(change(fields[9], None, ChangeKind::Unmerged));
            }
            "?" => status
                .untracked
                .push(change(rest, None, ChangeKind::Untracked)),
            "#" => {
                if read_header(&mut status.branch, rest, record)? {
                    has_ahead_behind = true;
                }
            }
            // Ignored files: not requested, skip if present.
            "!" => {}
            _ => return Err(unexpected(record)),
        }
    }
    // git omits "branch.ab" when the configured upstream does not exist any more.
    status.branch.upstream_gone = status.branch.upstream.is_some() && !has_ahead_behind;
    Ok(status)
}

/// Reads one "branch.<key> <value>" header into `branch`.
/// Returns true for the "branch.ab" (ahead/behind) header.
fn read_header(branch: &mut BranchInfo, header: &str, record: &str) -> Result<bool, AppError> {
    let (key, value) = header.split_once(' ').unwrap_or((header, ""));
    match key {
        "branch.oid" => {
            branch.commit = if value == "(initial)" {
                None
            } else {
                Some(value.to_string())
            };
        }
        "branch.head" => {
            branch.name = if value == "(detached)" {
                None
            } else {
                Some(value.to_string())
            };
        }
        "branch.upstream" => branch.upstream = Some(value.to_string()),
        "branch.ab" => {
            let (ahead, behind) = parse_ahead_behind(value).ok_or_else(|| unexpected(record))?;
            branch.ahead = ahead;
            branch.behind = behind;
            return Ok(true);
        }
        _ => {} // other headers (e.g. "# stash") are not used
    }
    Ok(false)
}

/// "+3 -1" -> (3, 1)
fn parse_ahead_behind(value: &str) -> Option<(u32, u32)> {
    let (ahead, behind) = value.split_once(' ')?;
    let ahead = ahead.strip_prefix('+')?.parse().ok()?;
    let behind = behind.strip_prefix('-')?.parse().ok()?;
    Some((ahead, behind))
}

/// Splits `rest` into exactly `count` space-separated fields. The last field
/// is the path and keeps its spaces.
fn split_fields<'a>(rest: &'a str, count: usize, record: &str) -> Result<Vec<&'a str>, AppError> {
    let fields: Vec<&str> = rest.splitn(count, ' ').collect();
    if fields.len() != count {
        return Err(unexpected(record));
    }
    Ok(fields)
}

/// Adds the index side (X) to `staged` and the working-tree side (Y) to `unstaged`.
fn push_changes(
    status: &mut RepoStatus,
    xy: &str,
    path: &str,
    orig_path: Option<&str>,
    record: &str,
) -> Result<(), AppError> {
    let mut chars = xy.chars();
    let (Some(x), Some(y), None) = (chars.next(), chars.next(), chars.next()) else {
        return Err(unexpected(record));
    };

    if x != '.' {
        let kind = kind_from_letter(x).ok_or_else(|| unexpected(record))?;
        status.staged.push(change(path, orig_path, kind));
    }
    if y != '.' {
        let kind = kind_from_letter(y).ok_or_else(|| unexpected(record))?;
        status.unstaged.push(change(path, orig_path, kind));
    }
    Ok(())
}

fn kind_from_letter(letter: char) -> Option<ChangeKind> {
    match letter {
        'A' => Some(ChangeKind::Added),
        'M' => Some(ChangeKind::Modified),
        'D' => Some(ChangeKind::Deleted),
        'R' => Some(ChangeKind::Renamed),
        'C' => Some(ChangeKind::Copied),
        'T' => Some(ChangeKind::TypeChanged),
        _ => None,
    }
}

fn change(path: &str, orig_path: Option<&str>, kind: ChangeKind) -> FileChange {
    // Only renames and copies have a meaningful original path.
    let orig_path = match kind {
        ChangeKind::Renamed | ChangeKind::Copied => orig_path.map(|p| p.to_string()),
        _ => None,
    };
    FileChange {
        path: path.to_string(),
        orig_path,
        kind,
    }
}

fn unexpected(record: &str) -> AppError {
    AppError::new(
        ErrorKind::UnexpectedOutput,
        format!("Could not read git status output: {record:?}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn file(path: &str, kind: ChangeKind) -> FileChange {
        FileChange {
            path: path.to_string(),
            orig_path: None,
            kind,
        }
    }

    // ---- parser, with hand-written output ----

    #[test]
    fn empty_output_means_clean() {
        assert_eq!(parse_status("").unwrap(), RepoStatus::default());
    }

    #[test]
    fn splits_index_and_worktree_sides() {
        let out = "1 MM N... 100644 100644 100644 aaa bbb src/a b.txt\0\
                   1 A. N... 000000 100644 100644 000 ccc new.txt\0\
                   1 .D N... 100644 100644 000000 ddd ddd gone.txt\0";
        let status = parse_status(out).unwrap();

        assert_eq!(
            status.staged,
            vec![
                file("src/a b.txt", ChangeKind::Modified),
                file("new.txt", ChangeKind::Added)
            ]
        );
        assert_eq!(
            status.unstaged,
            vec![
                file("src/a b.txt", ChangeKind::Modified),
                file("gone.txt", ChangeKind::Deleted)
            ]
        );
    }

    #[test]
    fn reads_rename_with_original_path() {
        let out = "2 R. N... 100644 100644 100644 aaa aaa R100 new name.txt\0old name.txt\0";
        let status = parse_status(out).unwrap();

        assert_eq!(
            status.staged,
            vec![FileChange {
                path: "new name.txt".to_string(),
                orig_path: Some("old name.txt".to_string()),
                kind: ChangeKind::Renamed,
            }]
        );
        assert!(status.unstaged.is_empty());
    }

    #[test]
    fn reads_conflicts_and_untracked() {
        let out = "u UU N... 100644 100644 100644 100644 h1 h2 h3 both.txt\0? dir/new file.txt\0";
        let status = parse_status(out).unwrap();

        assert_eq!(
            status.conflicted,
            vec![file("both.txt", ChangeKind::Unmerged)]
        );
        assert_eq!(
            status.untracked,
            vec![file("dir/new file.txt", ChangeKind::Untracked)]
        );
    }

    #[test]
    fn skips_ignored_files_and_unknown_headers() {
        let out = "# stash 3\0! target/\0";
        assert_eq!(parse_status(out).unwrap(), RepoStatus::default());
    }

    #[test]
    fn reads_branch_headers() {
        let out = "# branch.oid 1234abcd\0# branch.head main\0\
                   # branch.upstream origin/main\0# branch.ab +2 -5\0";
        assert_eq!(
            parse_status(out).unwrap().branch,
            BranchInfo {
                name: Some("main".to_string()),
                commit: Some("1234abcd".to_string()),
                upstream: Some("origin/main".to_string()),
                upstream_gone: false,
                ahead: 2,
                behind: 5,
            }
        );
    }

    #[test]
    fn reads_initial_detached_and_gone_branches() {
        let initial = parse_status("# branch.oid (initial)\0# branch.head main\0").unwrap();
        assert_eq!(initial.branch.commit, None);
        assert_eq!(initial.branch.name.as_deref(), Some("main"));
        assert_eq!(initial.branch.upstream, None);

        let detached = parse_status("# branch.oid abc\0# branch.head (detached)\0").unwrap();
        assert_eq!(detached.branch.name, None);

        // Upstream configured, but no "branch.ab": it was deleted on the server.
        let gone = parse_status("# branch.oid abc\0# branch.head x\0# branch.upstream origin/x\0")
            .unwrap();
        assert!(gone.branch.upstream_gone);
        assert_eq!((gone.branch.ahead, gone.branch.behind), (0, 0));
    }

    #[test]
    fn rejects_malformed_records() {
        for bad in [
            "x what\0",
            "1 M. too few fields\0",
            "1 MMM N... a b c d e path\0",
            "2 R. N... a b c d e R100 path\0",
            "# branch.ab 2 5\0",
        ] {
            let err = parse_status(bad).unwrap_err();
            assert_eq!(err.kind, ErrorKind::UnexpectedOutput, "input: {bad:?}");
        }
    }

    // ---- against a real repository ----

    // Runs git with a fixed identity, so commits work on any machine.
    fn git(dir: &Path, args: &[&str]) -> String {
        let mut all = vec!["-c", "user.name=Test", "-c", "user.email=test@example.com"];
        all.extend_from_slice(args);
        run_git(dir, &all).unwrap()
    }

    fn repo_with_commit() -> TempDir {
        let dir = TempDir::new().unwrap();
        git(dir.path(), &["init", "--quiet"]);
        fs::write(dir.path().join("keep.txt"), "keep\n").unwrap();
        fs::write(dir.path().join("edit.txt"), "one\n").unwrap();
        fs::write(dir.path().join("old.txt"), "rename me\n").unwrap();
        fs::write(dir.path().join("remove.txt"), "bye\n").unwrap();
        git(dir.path(), &["add", "."]);
        git(dir.path(), &["commit", "--quiet", "-m", "init"]);
        dir
    }

    #[test]
    fn clean_repository_has_no_changes() {
        let dir = repo_with_commit();
        let status = read_status(dir.path()).unwrap();
        assert_eq!(
            status,
            RepoStatus {
                branch: status.branch.clone(),
                ..RepoStatus::default()
            }
        );
        // The default branch name depends on the machine (main / master).
        assert!(status.branch.name.is_some());
        assert_eq!(status.branch.commit.as_ref().map(|c| c.len()), Some(40));
        assert_eq!(status.branch.upstream, None);
    }

    #[test]
    fn reports_real_changes() {
        let dir = repo_with_commit();
        let root = dir.path();
        fs::write(root.join("staged new.txt"), "new\n").unwrap();
        git(root, &["add", "staged new.txt"]);
        git(root, &["mv", "old.txt", "renamed.txt"]);
        fs::write(root.join("edit.txt"), "two\n").unwrap();
        fs::remove_file(root.join("remove.txt")).unwrap();
        fs::create_dir(root.join("fresh dir")).unwrap();
        fs::write(root.join("fresh dir").join("a.txt"), "a\n").unwrap();

        let status = read_status(root).unwrap();

        assert_eq!(
            status.staged,
            vec![
                FileChange {
                    path: "renamed.txt".to_string(),
                    orig_path: Some("old.txt".to_string()),
                    kind: ChangeKind::Renamed,
                },
                file("staged new.txt", ChangeKind::Added),
            ]
        );
        assert_eq!(
            status.unstaged,
            vec![
                file("edit.txt", ChangeKind::Modified),
                file("remove.txt", ChangeKind::Deleted)
            ]
        );
        // --untracked-files=all lists the file, not just the folder.
        assert_eq!(
            status.untracked,
            vec![file("fresh dir/a.txt", ChangeKind::Untracked)]
        );
        assert!(status.conflicted.is_empty());
    }

    #[test]
    fn reports_merge_conflict() {
        let dir = repo_with_commit();
        let root = dir.path();
        git(root, &["checkout", "--quiet", "-b", "other"]);
        fs::write(root.join("edit.txt"), "from other\n").unwrap();
        git(root, &["commit", "--quiet", "-am", "other"]);
        git(root, &["checkout", "--quiet", "-"]);
        fs::write(root.join("edit.txt"), "from main\n").unwrap();
        git(root, &["commit", "--quiet", "-am", "main"]);

        // The merge fails because of the conflict; that's the state we want.
        let mut args = vec!["-c", "user.name=Test", "-c", "user.email=test@example.com"];
        args.extend_from_slice(&["merge", "other"]);
        assert!(run_git(root, &args).is_err());

        let status = read_status(root).unwrap();
        assert_eq!(
            status.conflicted,
            vec![file("edit.txt", ChangeKind::Unmerged)]
        );
    }
}
