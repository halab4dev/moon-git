// Uploading local commits to the server. A normal push is never forced: if
// the server has commits we don't, git refuses and the user is told to pull
// first. Force push (step 27) is a separate, confirmed action, and always
// "with lease": it only overwrites what the user was shown.

use std::path::Path;

use crate::error::{AppError, ErrorKind};
use crate::git::merge::current_branch;
use crate::git::operations::{Operations, run_git_with_progress};
use crate::git::runner::run_git;
use crate::models::{ForcePushPreview, LostCommit};

/// Pushes the current branch.
///
/// `new_upstream`: None for a branch that already tracks one (plain
/// `git push`). Some(remote) for a branch that was never pushed: it is pushed
/// to `remote` under the same name and tracks it from then on.
pub fn push(
    ops: &Operations,
    op_id: &str,
    repo: &Path,
    new_upstream: Option<&str>,
    on_line: impl FnMut(&str),
) -> Result<(), AppError> {
    match new_upstream {
        None => run_git_with_progress(ops, op_id, repo, &["push", "--progress"], on_line),
        Some(remote) => {
            // Only names git already knows: a value like "--force" can never get through.
            if !list_remotes(repo)?.iter().any(|r| r == remote) {
                return Err(AppError::new(
                    ErrorKind::InvalidInput,
                    format!("There is no remote named \"{remote}\"."),
                ));
            }
            // HEAD = the current branch, pushed to a branch with the same name.
            let args = ["push", "--progress", "--set-upstream", remote, "HEAD"];
            run_git_with_progress(ops, op_id, repo, &args, on_line)
        }
    }
}

/// What force pushing the current branch to its upstream would do. Based on
/// the last fetch: commits pushed by others since then aren't known here,
/// which is why `force_push` checks `expected` on the server.
pub fn force_push_preview(repo: &Path) -> Result<ForcePushPreview, AppError> {
    let target = upstream_of_current(repo)?;
    let expected = tracking_commit(repo, &target)?;
    let count = |range: &str| -> Result<u32, AppError> {
        let out = run_git(repo, &["rev-list", "--count", range, "--"])?;
        Ok(out.trim().parse().unwrap_or(0))
    };
    let ahead = count(&format!("{expected}..HEAD"))?;
    let lost_count = count(&format!("HEAD..{expected}"))?;
    if ahead == 0 && lost_count == 0 {
        return Err(invalid(format!(
            "\"{}\" is the same as {}: there is nothing to push.",
            target.branch, target.short
        )));
    }

    let log = run_git(
        repo,
        &[
            "log",
            "-z",
            "--max-count=10",
            "--format=%H%x1f%s",
            &format!("HEAD..{expected}"),
            "--",
        ],
    )?;
    let lost = log
        .split('\0')
        .filter_map(|entry| entry.trim_start_matches('\n').split_once('\x1f'))
        .map(|(id, subject)| LostCommit {
            id: id.to_string(),
            subject: subject.to_string(),
        })
        .collect();

    Ok(ForcePushPreview {
        branch: target.branch,
        upstream: target.short,
        ahead,
        lost_count,
        lost,
        expected,
    })
}

/// Force pushes the current branch to its upstream, but only if the server
/// branch is still at `expected` (from `force_push_preview`). If someone
/// pushed in the meantime, git refuses and nothing is overwritten.
pub fn force_push(
    ops: &Operations,
    op_id: &str,
    repo: &Path,
    expected: &str,
    on_line: impl FnMut(&str),
) -> Result<(), AppError> {
    // A full id, so the lease can't be a ref name or an option.
    let full_id =
        matches!(expected.len(), 40 | 64) && expected.chars().all(|c| c.is_ascii_hexdigit());
    if !full_id {
        return Err(invalid(format!("Not a full commit id: {expected:?}")));
    }
    let target = upstream_of_current(repo)?;
    let lease = format!("--force-with-lease={}:{expected}", target.remote_ref);
    // No "+" in the refspec: the lease is what allows the overwrite.
    let refspec = format!("HEAD:{}", target.remote_ref);
    let args = ["push", "--progress", &lease, "--", &target.remote, &refspec];
    run_git_with_progress(ops, op_id, repo, &args, on_line)
}

/// The upstream of the current branch, as configured.
struct Upstream {
    branch: String,
    /// "origin".
    remote: String,
    /// The branch on the server: "refs/heads/main".
    remote_ref: String,
    /// The local copy of it: "refs/remotes/origin/main".
    tracking_ref: String,
    /// "origin/main".
    short: String,
}

fn upstream_of_current(repo: &Path) -> Result<Upstream, AppError> {
    let branch = current_branch(repo)?;
    let out = run_git(
        repo,
        &[
            "for-each-ref",
            "--format=%(upstream:remotename)%1f%(upstream:remoteref)%1f%(upstream)%1f%(upstream:short)",
            &format!("refs/heads/{branch}"),
        ],
    )?;
    let line = out.lines().next().unwrap_or("");
    let fields: Vec<&str> = line.split('\x1f').collect();
    match fields[..] {
        [remote, remote_ref, tracking_ref, short]
            if !remote.is_empty() && remote_ref.starts_with("refs/heads/") =>
        {
            Ok(Upstream {
                branch,
                remote: remote.to_string(),
                remote_ref: remote_ref.to_string(),
                tracking_ref: tracking_ref.to_string(),
                short: short.to_string(),
            })
        }
        _ => Err(invalid(format!(
            "\"{branch}\" has no upstream branch on a remote. Use Push to publish it first."
        ))),
    }
}

/// Where the server branch was at the last fetch.
fn tracking_commit(repo: &Path, target: &Upstream) -> Result<String, AppError> {
    let spec = format!("{}^{{commit}}", target.tracking_ref);
    run_git(repo, &["rev-parse", "--verify", "--quiet", &spec])
        .map(|out| out.trim().to_string())
        .map_err(|_| {
            invalid(format!(
                "{} doesn't exist (deleted on the server, or not fetched yet). Fetch first, or use Push to create it again.",
                target.short
            ))
        })
}

fn invalid(message: String) -> AppError {
    AppError::new(ErrorKind::InvalidInput, message)
}

/// Names of the configured remotes ("origin", ...), in git's order.
pub fn list_remotes(repo: &Path) -> Result<Vec<String>, AppError> {
    let out = run_git(repo, &["remote"])?;
    Ok(out
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::status::read_status;
    use crate::git::test_support::{clone_of, commit_file, git, server_and_clone};

    fn push_plain(dir: &Path) -> Result<(), AppError> {
        push(&Operations::new(), "op", dir, None, |_| {})
    }

    #[test]
    fn pushes_local_commits() {
        let (server, mine) = server_and_clone();
        commit_file(mine.path(), "mine.txt");
        assert_eq!(read_status(mine.path()).unwrap().branch.ahead, 1);

        push_plain(mine.path()).unwrap();

        assert_eq!(read_status(mine.path()).unwrap().branch.ahead, 0);
        let on_server = git(server.path(), &["log", "-1", "--format=%s", "main"]);
        assert_eq!(on_server.trim(), "mine.txt");
    }

    #[test]
    fn first_push_of_a_branch_sets_its_upstream() {
        let (server, mine) = server_and_clone();
        git(mine.path(), &["checkout", "--quiet", "-b", "feature"]);
        commit_file(mine.path(), "feature.txt");

        push(
            &Operations::new(),
            "op",
            mine.path(),
            Some("origin"),
            |_| {},
        )
        .unwrap();

        let branch = read_status(mine.path()).unwrap().branch;
        assert_eq!(branch.upstream.as_deref(), Some("origin/feature"));
        assert_eq!(branch.ahead, 0);
        git(
            server.path(),
            &["rev-parse", "--verify", "--quiet", "refs/heads/feature"],
        );
    }

    #[test]
    fn rejected_when_server_has_new_commits() {
        let (server, mine) = server_and_clone();
        let theirs = clone_of(server.path());
        commit_file(theirs.path(), "theirs.txt");
        git(theirs.path(), &["push", "--quiet", "origin", "main"]);
        commit_file(mine.path(), "mine.txt");

        let err = push_plain(mine.path()).unwrap_err();
        assert_eq!(err.kind, ErrorKind::PushRejected, "{}", err.message);
        // Nothing was overwritten on the server.
        let on_server = git(server.path(), &["log", "-1", "--format=%s", "main"]);
        assert_eq!(on_server.trim(), "theirs.txt");
    }

    #[test]
    fn unknown_remote_is_refused_before_running_git() {
        let (_server, mine) = server_and_clone();
        for remote in ["upstream", "--force"] {
            let err =
                push(&Operations::new(), "op", mine.path(), Some(remote), |_| {}).unwrap_err();
            assert_eq!(err.kind, ErrorKind::InvalidInput, "{remote}");
        }
    }

    fn force(dir: &Path, expected: &str) -> Result<(), AppError> {
        force_push(&Operations::new(), "op", dir, expected, |_| {})
    }

    fn server_log(server: &Path) -> String {
        git(server, &["log", "--format=%s", "main"])
    }

    /// The server has "theirs.txt" (fetched), and mine has rewritten history:
    /// "mine.txt" instead.
    fn diverged() -> (tempfile::TempDir, tempfile::TempDir) {
        let (server, mine) = server_and_clone();
        let theirs = clone_of(server.path());
        commit_file(theirs.path(), "theirs.txt");
        git(theirs.path(), &["push", "--quiet", "origin", "main"]);
        git(mine.path(), &["fetch", "--quiet"]);
        commit_file(mine.path(), "mine.txt");
        (server, mine)
    }

    #[test]
    fn preview_lists_what_the_server_loses() {
        let (_server, mine) = diverged();

        let preview = force_push_preview(mine.path()).unwrap();

        assert_eq!(preview.branch, "main");
        assert_eq!(preview.upstream, "origin/main");
        assert_eq!(preview.ahead, 1);
        assert_eq!(preview.lost_count, 1);
        assert_eq!(preview.lost.len(), 1);
        assert_eq!(preview.lost[0].subject, "theirs.txt");
        assert_eq!(
            preview.expected,
            git(mine.path(), &["rev-parse", "origin/main"]).trim()
        );
    }

    #[test]
    fn force_push_replaces_the_server_branch() {
        let (server, mine) = diverged();
        let preview = force_push_preview(mine.path()).unwrap();

        force(mine.path(), &preview.expected).unwrap();

        assert_eq!(server_log(server.path()), "mine.txt\nfirst.txt\n");
        let branch = read_status(mine.path()).unwrap().branch;
        assert_eq!((branch.ahead, branch.behind), (0, 0));
    }

    #[test]
    fn force_push_refuses_when_the_server_changed_since_the_preview() {
        let (server, mine) = diverged();
        let preview = force_push_preview(mine.path()).unwrap();
        // Someone else pushes after our last fetch.
        let other = clone_of(server.path());
        commit_file(other.path(), "late.txt");
        git(other.path(), &["push", "--quiet", "origin", "main"]);

        let err = force(mine.path(), &preview.expected).unwrap_err();

        assert_eq!(err.kind, ErrorKind::PushRejected, "{}", err.message);
        assert!(
            err.message.contains("since your last fetch"),
            "{}",
            err.message
        );
        // Their commit is still there.
        assert!(server_log(server.path()).starts_with("late.txt\n"));
    }

    #[test]
    fn force_push_needs_a_full_commit_id() {
        let (_server, mine) = diverged();
        for expected in ["", "abc123", "--force", "refs/heads/main"] {
            let err = force(mine.path(), expected).unwrap_err();
            assert_eq!(err.kind, ErrorKind::InvalidInput, "{expected}");
        }
    }

    #[test]
    fn preview_refuses_without_upstream_or_changes() {
        let (_server, mine) = server_and_clone();
        // Same as the server: nothing to do.
        let err = force_push_preview(mine.path()).unwrap_err();
        assert!(err.message.contains("nothing to push"), "{}", err.message);

        git(
            mine.path(),
            &["switch", "--quiet", "--create", "local-only"],
        );
        let err = force_push_preview(mine.path()).unwrap_err();
        assert!(err.message.contains("no upstream"), "{}", err.message);
    }

    #[test]
    fn lists_remotes() {
        let (_server, mine) = server_and_clone();
        git(mine.path(), &["remote", "add", "backup", "/nowhere"]);
        assert_eq!(list_remotes(mine.path()).unwrap(), vec!["backup", "origin"]);
    }
}
