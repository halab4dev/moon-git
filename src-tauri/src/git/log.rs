// Reads the commit history with `git log` in a machine-readable format.
//
// One record per commit, records separated by NUL (-z), fields by 0x1f:
//   <id> 0x1f <parent ids, space separated> 0x1f <author> 0x1f <email>
//   0x1f <author time, unix seconds> 0x1f <decorations> 0x1f <subject>
// The subject is last: it is free text, so whatever it contains ends up in
// the last field instead of shifting the others.

use std::path::Path;

use crate::error::{AppError, ErrorKind};
use crate::git::repo::has_commits;
use crate::git::runner::run_git;
use crate::models::{CommitRef, CommitSummary, LogPage, RefKind};

const FORMAT: &str = "--format=%H%x1f%P%x1f%aN%x1f%aE%x1f%at%x1f%D%x1f%s";
const FIELDS: usize = 7;

/// Largest page the frontend may ask for, so one call stays fast.
pub const MAX_PAGE: usize = 2000;

/// `limit` commits starting after the first `skip`, newest first, from all
/// local branches, remote-tracking branches, tags and HEAD.
pub fn read_log(repo: &Path, skip: usize, limit: usize) -> Result<LogPage, AppError> {
    if limit == 0 || limit > MAX_PAGE {
        return Err(AppError::new(
            ErrorKind::InvalidInput,
            format!("Page size must be between 1 and {MAX_PAGE}."),
        ));
    }
    // One extra commit tells us whether another page exists.
    let max_count = format!("--max-count={}", limit + 1);
    let skip_arg = format!("--skip={skip}");

    let mut args = vec![
        "log",
        "-z",
        FORMAT,
        // Full ref names ("refs/heads/main") say what kind of ref each one is.
        "--decorate=full",
        // A commit is listed only after all of its children, newest first:
        // what the graph (step 16b) needs to draw lines downwards.
        "--date-order",
        // Neutralise user settings that would add text to the output.
        "--no-show-signature",
        "--no-color",
        &max_count,
        &skip_arg,
        // Not --all: that would also list stash commits.
        "--branches",
        "--remotes",
        "--tags",
    ];
    // HEAD matters when it is detached; in an empty repository it names nothing.
    if has_commits(repo) {
        args.push("HEAD");
    }
    // End of revisions: nothing after this may be taken for a file name.
    args.push("--");

    let output = run_git(repo, &args)?;
    let mut commits = parse_log(&output)?;
    let has_more = commits.len() > limit;
    commits.truncate(limit);
    Ok(LogPage { commits, has_more })
}

pub fn parse_log(output: &str) -> Result<Vec<CommitSummary>, AppError> {
    let mut commits = Vec::new();
    for record in output.split('\0') {
        // git separates -z records with NUL, but may still put a newline
        // between them; neither belongs to the data.
        let record = record.trim_start_matches('\n');
        if record.is_empty() {
            continue;
        }
        commits.push(parse_record(record)?);
    }
    Ok(commits)
}

fn parse_record(record: &str) -> Result<CommitSummary, AppError> {
    let fields: Vec<&str> = record.splitn(FIELDS, '\x1f').collect();
    let [id, parents, name, email, time, decorations, subject] = fields[..] else {
        return Err(unexpected(record));
    };
    if id.is_empty() {
        return Err(unexpected(record));
    }
    let author_time = time.parse::<i64>().map_err(|_| unexpected(record))?;

    Ok(CommitSummary {
        id: id.to_string(),
        parents: parents.split_whitespace().map(|p| p.to_string()).collect(),
        author_name: name.to_string(),
        author_email: email.to_string(),
        author_time,
        subject: subject.to_string(),
        refs: parse_decorations(decorations),
    })
}

/// "HEAD -> refs/heads/main, refs/remotes/origin/main, tag: refs/tags/v1"
/// Ref names cannot contain spaces, so ", " safely separates them.
fn parse_decorations(text: &str) -> Vec<CommitRef> {
    let mut refs = Vec::new();
    for item in text.split(", ").filter(|item| !item.is_empty()) {
        if let Some(branch) = item.strip_prefix("HEAD -> ") {
            // HEAD on a branch: mark that branch as the current one.
            if let Some(found) = classify(branch, true) {
                refs.push(found);
            }
        } else if item == "HEAD" {
            refs.push(CommitRef {
                kind: RefKind::Head,
                name: "HEAD".to_string(),
                current: true,
            });
        } else if let Some(tag) = item.strip_prefix("tag: ") {
            refs.extend(classify(tag, false));
        } else {
            refs.extend(classify(item, false));
        }
    }
    refs
}

// Full ref name -> kind + short name. Other refs (stash, notes, "origin/HEAD",
// which only says which branch is the server's default) are left out.
fn classify(full_name: &str, current: bool) -> Option<CommitRef> {
    const PREFIXES: [(&str, RefKind); 3] = [
        ("refs/heads/", RefKind::Branch),
        ("refs/remotes/", RefKind::Remote),
        ("refs/tags/", RefKind::Tag),
    ];
    for (prefix, kind) in PREFIXES {
        let Some(name) = full_name.strip_prefix(prefix) else {
            continue;
        };
        if kind == RefKind::Remote && name.ends_with("/HEAD") {
            return None;
        }
        return Some(CommitRef {
            kind,
            name: name.to_string(),
            current,
        });
    }
    None
}

fn unexpected(record: &str) -> AppError {
    AppError::new(
        ErrorKind::UnexpectedOutput,
        format!("Could not read git log output: {record:?}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_support::{commit_file, git, server_and_clone};
    use tempfile::TempDir;

    fn branch(name: &str, current: bool) -> CommitRef {
        CommitRef {
            kind: RefKind::Branch,
            name: name.to_string(),
            current,
        }
    }

    fn new_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        git(dir.path(), &["init", "--quiet", "--initial-branch=main"]);
        git(dir.path(), &["config", "user.name", "Test"]);
        git(dir.path(), &["config", "user.email", "test@example.com"]);
        dir
    }

    fn subjects(page: &LogPage) -> Vec<&str> {
        page.commits.iter().map(|c| c.subject.as_str()).collect()
    }

    // ---- parser, with hand-written output ----

    #[test]
    fn parses_records_and_decorations() {
        let out = "aaa\x1fbbb ccc\x1fAnn\x1fann@x.org\x1f1700000000\x1f\
                   HEAD -> refs/heads/main, refs/remotes/origin/main, refs/remotes/origin/HEAD, tag: refs/tags/v1.0\x1f\
                   Merge branch 'x'\0\
                   \nbbb\x1f\x1fBob\x1fbob@x.org\x1f1690000000\x1f\x1fFirst: a\x1fb\0";
        let commits = parse_log(out).unwrap();

        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].parents, vec!["bbb", "ccc"]);
        assert_eq!(commits[0].author_time, 1_700_000_000);
        assert_eq!(
            commits[0].refs,
            vec![
                branch("main", true),
                CommitRef {
                    kind: RefKind::Remote,
                    name: "origin/main".to_string(),
                    current: false
                },
                CommitRef {
                    kind: RefKind::Tag,
                    name: "v1.0".to_string(),
                    current: false
                },
            ]
        );
        // Root commit: no parents, no refs; a 0x1f inside the subject is kept.
        assert!(commits[1].parents.is_empty());
        assert!(commits[1].refs.is_empty());
        assert_eq!(commits[1].subject, "First: a\x1fb");
    }

    #[test]
    fn detached_head_and_slashes_in_names() {
        let refs = parse_decorations("HEAD, refs/heads/feature/login, refs/stash");
        assert_eq!(
            refs,
            vec![
                CommitRef {
                    kind: RefKind::Head,
                    name: "HEAD".to_string(),
                    current: true
                },
                branch("feature/login", false),
            ]
        );
    }

    #[test]
    fn rejects_malformed_records() {
        for bad in [
            "only-an-id\0",
            "aaa\x1f\x1fA\x1fa@x\x1fnot-a-time\x1f\x1fs\0",
        ] {
            let err = parse_log(bad).unwrap_err();
            assert_eq!(err.kind, ErrorKind::UnexpectedOutput, "input: {bad:?}");
        }
    }

    // ---- against a real repository ----

    #[test]
    fn empty_repository_has_no_history() {
        let dir = new_repo();
        let page = read_log(dir.path(), 0, 100).unwrap();
        assert!(page.commits.is_empty());
        assert!(!page.has_more);
    }

    #[test]
    fn lists_newest_first_with_paging() {
        let dir = new_repo();
        for name in ["one", "two", "three", "four", "five"] {
            commit_file(dir.path(), name);
        }

        let first = read_log(dir.path(), 0, 2).unwrap();
        assert_eq!(subjects(&first), vec!["five", "four"]);
        assert!(first.has_more);
        assert_eq!(first.commits[0].refs, vec![branch("main", true)]);
        assert_eq!(first.commits[0].parents, vec![first.commits[1].id.clone()]);

        let last = read_log(dir.path(), 4, 2).unwrap();
        assert_eq!(subjects(&last), vec!["one"]);
        assert!(!last.has_more);
        assert!(last.commits[0].parents.is_empty());
    }

    #[test]
    fn includes_other_branches_tags_and_merges() {
        let dir = new_repo();
        let root = dir.path();
        commit_file(root, "base");
        git(root, &["tag", "v1"]);
        git(root, &["checkout", "--quiet", "-b", "side"]);
        commit_file(root, "on-side");
        git(root, &["checkout", "--quiet", "main"]);
        commit_file(root, "on-main");
        git(root, &["merge", "--quiet", "--no-edit", "--no-ff", "side"]);

        let page = read_log(root, 0, 100).unwrap();
        let merge = &page.commits[0];
        assert_eq!(merge.parents.len(), 2);
        assert_eq!(merge.refs, vec![branch("main", true)]);

        let side = page
            .commits
            .iter()
            .find(|c| c.subject == "on-side")
            .unwrap();
        assert_eq!(side.refs, vec![branch("side", false)]);
        let base = page.commits.iter().find(|c| c.subject == "base").unwrap();
        assert_eq!(base.refs[0].kind, RefKind::Tag);

        // --date-order: every commit comes before its parents.
        for (index, commit) in page.commits.iter().enumerate() {
            for parent in &commit.parents {
                let parent_index = page.commits.iter().position(|c| &c.id == parent).unwrap();
                assert!(parent_index > index, "{} listed before its child", parent);
            }
        }
    }

    #[test]
    fn shows_remote_branches_and_detached_head() {
        let (_server, work) = server_and_clone();
        let root = work.path();
        commit_file(root, "local");
        git(root, &["checkout", "--quiet", "--detach"]);

        let page = read_log(root, 0, 10).unwrap();
        let kinds: Vec<RefKind> = page.commits[0].refs.iter().map(|r| r.kind).collect();
        assert_eq!(kinds, vec![RefKind::Head, RefKind::Branch]);
        let pushed = &page.commits[1];
        assert_eq!(pushed.refs[0].name, "origin/main");
    }

    #[test]
    fn rejects_bad_page_size() {
        let dir = new_repo();
        assert_eq!(
            read_log(dir.path(), 0, 0).unwrap_err().kind,
            ErrorKind::InvalidInput
        );
        assert_eq!(
            read_log(dir.path(), 0, MAX_PAGE + 1).unwrap_err().kind,
            ErrorKind::InvalidInput
        );
    }
}
