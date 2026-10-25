// Branches: checking out and creating (step 19a), renaming, deleting and
// changing the upstream (step 19b).
//
// Never forced on its own: when local changes are in the way, git refuses
// and its message ("commit or stash them first") is shown as is. Deleting an
// unmerged branch needs a second, explicit confirmation in the UI (`force`).

use std::path::Path;

use crate::error::{AppError, ErrorKind};
use crate::git::operations::{Operations, run_git_with_progress};
use crate::git::push::list_remotes;
use crate::git::refs::list_refs;
use crate::git::runner::run_git;
use crate::git::show::check_commit_id;
use crate::models::{RefInfo, RefKind};

/// Checks out a local branch, or, for a remote branch ("refs/remotes/origin/x"),
/// a local branch "x" that tracks it (created if needed).
pub fn checkout(repo: &Path, full_name: &str) -> Result<(), AppError> {
    let refs = list_refs(repo)?;
    let target = find(&refs, full_name)?;
    match target.kind {
        RefKind::Branch => switch(repo, &["--no-guess", "--end-of-options", &target.name]),
        RefKind::Remote => checkout_remote(repo, &refs, target),
        _ => Err(invalid(format!("{} is not a branch.", target.name))),
    }
}

fn checkout_remote(repo: &Path, refs: &[RefInfo], remote: &RefInfo) -> Result<(), AppError> {
    // "origin/feature/login" -> "feature/login"
    let local = remote
        .name
        .split_once('/')
        .map(|(_, rest)| rest)
        .unwrap_or(&remote.name);
    let local_full = format!("refs/heads/{local}");

    if let Some(existing) = refs.iter().find(|r| r.full_name == local_full) {
        // Already have it: only switch if it is really "the same" branch.
        if existing.upstream.as_deref() == Some(remote.full_name.as_str()) {
            return switch(repo, &["--no-guess", "--end-of-options", local]);
        }
        return Err(invalid(format!(
            "A local branch \"{local}\" already exists and does not track {}. \
             Check it out directly, or rename it first.",
            remote.name
        )));
    }
    // New local branch at the remote branch, tracking it.
    switch(
        repo,
        &[
            "--create",
            local,
            "--track",
            "--end-of-options",
            &remote.full_name,
        ],
    )
}

/// Creates branch `name` at `start` (a commit id or a full ref name from
/// `list_refs`; None = the current HEAD), and checks it out if `check_out`.
/// Starting from a remote branch makes the new branch track it.
pub fn create(
    repo: &Path,
    name: &str,
    start: Option<&str>,
    check_out: bool,
) -> Result<(), AppError> {
    let name = check_branch_name(repo, name)?;
    let refs = list_refs(repo)?;
    if refs
        .iter()
        .any(|r| r.full_name == format!("refs/heads/{name}"))
    {
        return Err(invalid(format!(
            "A branch named \"{name}\" already exists."
        )));
    }

    let (start_point, track) = match start {
        None => ("HEAD".to_string(), false),
        Some(start) if start.starts_with("refs/") => {
            let found = find(&refs, start)?;
            (found.full_name.clone(), found.kind == RefKind::Remote)
        }
        Some(commit) => {
            check_commit_id(commit)?;
            (commit.to_string(), false)
        }
    };
    let track_flag = if track { "--track" } else { "--no-track" };

    if check_out {
        switch(
            repo,
            &[
                "--create",
                &name,
                track_flag,
                "--end-of-options",
                &start_point,
            ],
        )
    } else {
        run_git(
            repo,
            &[
                "branch",
                track_flag,
                "--end-of-options",
                &name,
                &start_point,
            ],
        )?;
        Ok(())
    }
}

/// Renames local branch `full_name` ("refs/heads/x") to `new_name`. Works for
/// the current branch too. Its upstream setting moves along; the branch on
/// the server keeps its old name.
pub fn rename(repo: &Path, full_name: &str, new_name: &str) -> Result<(), AppError> {
    let refs = list_refs(repo)?;
    let branch = find_local(&refs, full_name)?;
    let new_name = check_branch_name(repo, new_name)?;
    if new_name == branch.name {
        return Ok(());
    }
    if refs
        .iter()
        .any(|r| r.full_name == format!("refs/heads/{new_name}"))
    {
        return Err(invalid(format!(
            "A branch named \"{new_name}\" already exists."
        )));
    }
    run_git(
        repo,
        &[
            "branch",
            "--move",
            "--end-of-options",
            &branch.name,
            &new_name,
        ],
    )?;
    Ok(())
}

/// Deletes local branch `full_name`. Without `force`, git refuses when the
/// branch has commits no other branch contains (ErrorKind::NotMerged), so the
/// UI can ask again before `force` throws them away.
pub fn delete_local(repo: &Path, full_name: &str, force: bool) -> Result<(), AppError> {
    let refs = list_refs(repo)?;
    let branch = find_local(&refs, full_name)?;
    if branch.current {
        return Err(invalid(format!(
            "\"{}\" is checked out. Check out another branch first.",
            branch.name
        )));
    }
    // --delete --force is "git branch -D".
    let mut args = vec!["branch", "--delete"];
    if force {
        args.push("--force");
    }
    args.extend(["--end-of-options", branch.name.as_str()]);
    run_git(repo, &args).map_err(|err| {
        if err.message.contains("not fully merged") {
            AppError::new(
                ErrorKind::NotMerged,
                format!(
                    "\"{}\" has commits that are not in any other branch. Deleting it anyway \
                     makes them hard to find (git keeps them for a while in its reflog).",
                    branch.name
                ),
            )
        } else {
            err
        }
    })?;
    Ok(())
}

/// Deletes a branch on the server: `full_name` is its remote-tracking ref
/// ("refs/remotes/origin/feature"). Runs `git push <remote> --delete`, which
/// also removes the local "origin/feature".
pub fn delete_remote(
    ops: &Operations,
    op_id: &str,
    repo: &Path,
    full_name: &str,
    on_line: impl FnMut(&str),
) -> Result<(), AppError> {
    let refs = list_refs(repo)?;
    let target = find(&refs, full_name)?;
    if target.kind != RefKind::Remote {
        return Err(invalid(format!("{} is not a remote branch.", target.name)));
    }
    let (remote, branch) = split_remote_name(&list_remotes(repo)?, &target.name)
        .ok_or_else(|| invalid(format!("No remote found for {}.", target.name)))?;
    // Full ref name on the server, so a tag with the same name is never hit.
    let server_ref = format!("refs/heads/{branch}");
    run_git_with_progress(
        ops,
        op_id,
        repo,
        &["push", "--progress", "--delete", &remote, &server_ref],
        on_line,
    )
}

/// "origin/feature/x" -> ("origin", "feature/x"). The longest remote name that
/// fits wins, because remote names may contain "/" too ("team/origin").
pub(crate) fn split_remote_name(remotes: &[String], short_name: &str) -> Option<(String, String)> {
    remotes
        .iter()
        .filter_map(|remote| {
            let branch = short_name
                .strip_prefix(remote.as_str())?
                .strip_prefix('/')?;
            Some((remote.clone(), branch.to_string()))
        })
        .max_by_key(|(remote, _)| remote.len())
}

/// Makes local branch `full_name` track remote branch `upstream`
/// ("refs/remotes/origin/x"), or stop tracking anything when `upstream` is None.
pub fn set_upstream(repo: &Path, full_name: &str, upstream: Option<&str>) -> Result<(), AppError> {
    let refs = list_refs(repo)?;
    let branch = find_local(&refs, full_name)?;
    match upstream {
        Some(upstream) => {
            let target = find(&refs, upstream)?;
            if target.kind != RefKind::Remote {
                return Err(invalid(format!("{} is not a remote branch.", target.name)));
            }
            let flag = format!("--set-upstream-to={}", target.full_name);
            run_git(repo, &["branch", &flag, "--end-of-options", &branch.name])?;
        }
        None if branch.upstream.is_none() => {} // nothing to remove
        None => {
            run_git(
                repo,
                &[
                    "branch",
                    "--unset-upstream",
                    "--end-of-options",
                    &branch.name,
                ],
            )?;
        }
    }
    Ok(())
}

/// Checks a new branch name with git's own rules and returns it trimmed.
fn check_branch_name(repo: &Path, name: &str) -> Result<String, AppError> {
    let name = name.trim();
    let problem = if name.is_empty() {
        Some("Enter a branch name.")
    } else if name.starts_with('-') {
        Some("A branch name cannot start with \"-\".")
    } else if name == "HEAD" {
        Some("\"HEAD\" cannot be used as a branch name.")
    } else {
        None
    };
    if let Some(problem) = problem {
        return Err(invalid(problem.to_string()));
    }
    // Spaces, "..", "~", "^", ":", a trailing ".lock" and more are not allowed.
    run_git(repo, &["check-ref-format", "--branch", name]).map_err(|_| {
        invalid(format!(
            "\"{name}\" is not a valid branch name (no spaces, \"..\", \"~\", \"^\", \":\", \"?\", \"*\" or \"[\")."
        ))
    })?;
    Ok(name.to_string())
}

fn switch(repo: &Path, args: &[&str]) -> Result<(), AppError> {
    let mut all = vec!["switch", "--quiet"];
    all.extend_from_slice(args);
    run_git(repo, &all).map_err(explain_switch_error)?;
    Ok(())
}

// git's "Your local changes to the following files would be overwritten by
// checkout:\n\ta.txt\n…\nPlease commit … Aborting" in the app's words: what
// is in the way and what to do, without the file list (the Changes tab shows
// the files; a long list would only make the message hard to read).
fn explain_switch_error(err: AppError) -> AppError {
    if !err.message.contains("would be overwritten") {
        return err;
    }
    // The files are the lines starting with a tab.
    let count = err
        .message
        .lines()
        .filter(|line| line.starts_with('\t'))
        .count();
    let files = if count == 1 {
        "1 file".to_string()
    } else {
        format!("{count} files")
    };
    invalid(if err.message.contains("untracked working tree files") {
        format!(
            "The branch can't be checked out: it contains {files} that already exist here as \
             untracked files. Move or delete them first."
        )
    } else {
        format!(
            "The branch can't be checked out: it changes {files} that you have uncommitted \
             changes in. Commit, stash or discard your changes first."
        )
    })
}

fn find<'a>(refs: &'a [RefInfo], full_name: &str) -> Result<&'a RefInfo, AppError> {
    refs.iter()
        .find(|r| r.full_name == full_name)
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::NotFound,
                format!("{full_name} does not exist (any more)."),
            )
        })
}

fn find_local<'a>(refs: &'a [RefInfo], full_name: &str) -> Result<&'a RefInfo, AppError> {
    let found = find(refs, full_name)?;
    if found.kind != RefKind::Branch {
        return Err(invalid(format!("{} is not a local branch.", found.name)));
    }
    Ok(found)
}

fn invalid(message: String) -> AppError {
    AppError::new(ErrorKind::InvalidInput, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::status::read_status;
    use crate::git::test_support::{clone_of, commit_file, git, server_and_clone};
    use std::fs;

    fn current_branch(dir: &Path) -> Option<String> {
        read_status(dir).unwrap().branch.name
    }

    fn upstream(dir: &Path) -> Option<String> {
        read_status(dir).unwrap().branch.upstream
    }

    #[test]
    fn checks_out_a_local_branch() {
        let (_server, work) = server_and_clone();
        git(work.path(), &["branch", "feature"]);

        checkout(work.path(), "refs/heads/feature").unwrap();
        assert_eq!(current_branch(work.path()).as_deref(), Some("feature"));
    }

    #[test]
    fn checking_out_a_remote_branch_creates_a_tracking_branch() {
        let (server, work) = server_and_clone();
        let other = clone_of(server.path());
        git(
            other.path(),
            &["checkout", "--quiet", "-b", "feature/login"],
        );
        commit_file(other.path(), "login.txt");
        git(
            other.path(),
            &["push", "--quiet", "origin", "feature/login"],
        );
        git(work.path(), &["fetch", "--quiet"]);

        checkout(work.path(), "refs/remotes/origin/feature/login").unwrap();
        assert_eq!(
            current_branch(work.path()).as_deref(),
            Some("feature/login")
        );
        assert_eq!(
            upstream(work.path()).as_deref(),
            Some("origin/feature/login")
        );
        assert!(work.path().join("login.txt").exists());

        // Second time: the tracking branch exists, so it is just checked out.
        git(work.path(), &["checkout", "--quiet", "main"]);
        checkout(work.path(), "refs/remotes/origin/feature/login").unwrap();
        assert_eq!(
            current_branch(work.path()).as_deref(),
            Some("feature/login")
        );
    }

    #[test]
    fn remote_checkout_refuses_an_unrelated_local_branch_with_that_name() {
        let (server, work) = server_and_clone();
        let other = clone_of(server.path());
        git(other.path(), &["push", "--quiet", "origin", "main:feature"]);
        git(work.path(), &["fetch", "--quiet"]);
        git(work.path(), &["branch", "--no-track", "feature"]);

        let err = checkout(work.path(), "refs/remotes/origin/feature").unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
        assert_eq!(current_branch(work.path()).as_deref(), Some("main"));
    }

    #[test]
    fn local_changes_in_the_way_stop_the_checkout() {
        let (_server, work) = server_and_clone();
        let root = work.path();
        git(root, &["checkout", "--quiet", "-b", "other"]);
        fs::write(root.join("first.txt"), "changed on other\n").unwrap();
        git(root, &["commit", "--quiet", "-am", "other"]);
        git(root, &["checkout", "--quiet", "main"]);
        fs::write(root.join("first.txt"), "my unsaved work\n").unwrap();

        let err = checkout(root, "refs/heads/other").unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
        assert!(
            err.message.starts_with(
                "The branch can't be checked out: it changes 1 file that you have uncommitted changes in."
            ),
            "{}",
            err.message
        );
        // Nothing lost.
        assert_eq!(
            fs::read_to_string(root.join("first.txt")).unwrap(),
            "my unsaved work\n"
        );
        assert_eq!(current_branch(root).as_deref(), Some("main"));
    }

    #[test]
    fn creates_branches_from_head_commit_or_remote() {
        let (_server, work) = server_and_clone();
        let root = work.path();
        let first = git(root, &["rev-parse", "HEAD"]).trim().to_string();
        commit_file(root, "second.txt");

        // From HEAD, without checking it out.
        create(root, "from-head", None, false).unwrap();
        assert_eq!(current_branch(root).as_deref(), Some("main"));
        assert_eq!(
            git(root, &["rev-parse", "from-head"]),
            git(root, &["rev-parse", "HEAD"])
        );

        // From an older commit, checked out.
        create(root, "old", Some(&first), true).unwrap();
        assert_eq!(current_branch(root).as_deref(), Some("old"));
        assert!(!root.join("second.txt").exists());
        assert_eq!(upstream(root), None);

        // From a remote branch: tracks it.
        create(root, "mirror", Some("refs/remotes/origin/main"), true).unwrap();
        assert_eq!(upstream(root).as_deref(), Some("origin/main"));
    }

    fn local_branches(dir: &Path) -> Vec<String> {
        list_refs(dir)
            .unwrap()
            .into_iter()
            .filter(|r| r.kind == RefKind::Branch)
            .map(|r| r.name)
            .collect()
    }

    #[test]
    fn renames_branches_including_the_current_one() {
        let (_server, work) = server_and_clone();
        let root = work.path();
        git(root, &["branch", "old-name"]);

        rename(root, "refs/heads/old-name", "new-name").unwrap();
        assert_eq!(local_branches(root), vec!["main", "new-name"]);

        // The checked-out branch keeps its upstream after a rename.
        rename(root, "refs/heads/main", "trunk").unwrap();
        assert_eq!(current_branch(root).as_deref(), Some("trunk"));
        assert_eq!(upstream(root).as_deref(), Some("origin/main"));

        let err = rename(root, "refs/heads/trunk", "new-name").unwrap_err();
        assert!(err.message.contains("already exists"), "{}", err.message);
        assert_eq!(
            rename(root, "refs/heads/trunk", "bad name")
                .unwrap_err()
                .kind,
            ErrorKind::InvalidInput
        );
        assert_eq!(
            rename(root, "refs/remotes/origin/main", "x")
                .unwrap_err()
                .kind,
            ErrorKind::InvalidInput
        );
    }

    #[test]
    fn deleting_an_unmerged_branch_needs_force() {
        let (_server, work) = server_and_clone();
        let root = work.path();
        git(root, &["branch", "merged"]);
        git(root, &["checkout", "--quiet", "-b", "unmerged"]);
        commit_file(root, "only-here.txt");
        git(root, &["checkout", "--quiet", "main"]);

        delete_local(root, "refs/heads/merged", false).unwrap();
        let err = delete_local(root, "refs/heads/unmerged", false).unwrap_err();
        assert_eq!(err.kind, ErrorKind::NotMerged, "{}", err.message);
        assert_eq!(local_branches(root), vec!["main", "unmerged"]);

        delete_local(root, "refs/heads/unmerged", true).unwrap();
        assert_eq!(local_branches(root), vec!["main"]);
    }

    #[test]
    fn cannot_delete_the_current_branch() {
        let (_server, work) = server_and_clone();
        let err = delete_local(work.path(), "refs/heads/main", true).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
        assert_eq!(local_branches(work.path()), vec!["main"]);
    }

    #[test]
    fn deletes_a_branch_on_the_server() {
        let (server, work) = server_and_clone();
        let root = work.path();
        git(root, &["push", "--quiet", "origin", "main:feature"]);
        git(root, &["fetch", "--quiet"]);

        delete_remote(
            &Operations::new(),
            "op",
            root,
            "refs/remotes/origin/feature",
            |_| {},
        )
        .unwrap();

        let on_server = git(server.path(), &["branch", "--list"]);
        assert!(!on_server.contains("feature"), "{on_server}");
        // The local remote-tracking ref is gone too.
        assert!(
            list_refs(root)
                .unwrap()
                .iter()
                .all(|r| r.name != "origin/feature")
        );
        // Only remote branches can be deleted this way.
        let err =
            delete_remote(&Operations::new(), "op", root, "refs/heads/main", |_| {}).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
    }

    #[test]
    fn splits_remote_names_with_slashes() {
        let remotes = vec!["origin".to_string(), "team/origin".to_string()];
        assert_eq!(
            split_remote_name(&remotes, "team/origin/feature/x"),
            Some(("team/origin".to_string(), "feature/x".to_string()))
        );
        assert_eq!(
            split_remote_name(&remotes, "origin/main"),
            Some(("origin".to_string(), "main".to_string()))
        );
        assert_eq!(split_remote_name(&remotes, "elsewhere/main"), None);
    }

    #[test]
    fn sets_and_removes_the_upstream() {
        let (_server, work) = server_and_clone();
        let root = work.path();
        git(root, &["push", "--quiet", "origin", "main:other"]);
        git(root, &["fetch", "--quiet"]);

        set_upstream(root, "refs/heads/main", Some("refs/remotes/origin/other")).unwrap();
        assert_eq!(upstream(root).as_deref(), Some("origin/other"));

        set_upstream(root, "refs/heads/main", None).unwrap();
        assert_eq!(upstream(root), None);
        // Removing again is fine.
        set_upstream(root, "refs/heads/main", None).unwrap();

        let err = set_upstream(root, "refs/heads/main", Some("refs/heads/main")).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
    }

    #[test]
    fn rejects_bad_or_existing_names() {
        let (_server, work) = server_and_clone();
        for bad in [
            "",
            "  ",
            "-x",
            "HEAD",
            "has space",
            "a..b",
            "x~1",
            "end.lock",
        ] {
            let err = create(work.path(), bad, None, false).unwrap_err();
            assert_eq!(err.kind, ErrorKind::InvalidInput, "{bad:?}");
        }
        let err = create(work.path(), "main", None, false).unwrap_err();
        assert!(err.message.contains("already exists"), "{}", err.message);
        // A start point must be a commit id or an existing ref.
        assert!(create(work.path(), "x", Some("--orphan"), false).is_err());
        assert!(create(work.path(), "x", Some("refs/heads/nope"), false).is_err());
    }
}
