// Branches: checking out and creating (step 19a). Renaming, deleting and
// changing the upstream come in step 19b.
//
// Never forced: when local changes are in the way, git refuses and its
// message ("commit or stash them first") is shown as is.

use std::path::Path;

use crate::error::{AppError, ErrorKind};
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
    run_git(repo, &all)?;
    Ok(())
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
        assert_eq!(err.kind, ErrorKind::GitFailed);
        assert!(err.message.contains("local changes"), "{}", err.message);
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
