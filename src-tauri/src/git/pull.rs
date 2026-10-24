// The second half of a pull: combining the fetched upstream commits with the
// local branch. The frontend runs fetch first (with progress and Cancel), then
// looks at ahead/behind to decide whether to ask "rebase or merge?".
//
// Done with local commands (`merge` / `rebase` against @{upstream}) instead of
// `git pull`, so the network part happens only once, in the fetch.

use std::path::Path;

use crate::error::{AppError, ErrorKind};
use crate::git::commit::explain_commit_error;
use crate::git::runner::run_git;
use crate::git::status::read_status;
use crate::models::PullMode;

pub fn pull(repo: &Path, mode: PullMode) -> Result<(), AppError> {
    let status = read_status(repo)?;
    let branch = &status.branch;
    if branch.name.is_none() {
        return Err(invalid("HEAD is detached. Check out a branch to pull."));
    }
    if branch.upstream.is_none() {
        return Err(invalid(
            "This branch has no upstream branch on a server, so there is nothing to pull from.",
        ));
    }
    if branch.upstream_gone {
        return Err(invalid("The upstream branch was deleted on the server."));
    }
    if !status.conflicted.is_empty() {
        return Err(invalid(
            "Some files still have conflicts. Resolve them before pulling.",
        ));
    }

    let args: &[&str] = match mode {
        PullMode::FastForward => &["merge", "--ff-only", "@{upstream}"],
        // --no-edit: use git's default message, don't open an editor.
        PullMode::Merge => &["merge", "--no-edit", "@{upstream}"],
        // --fork-point: what `git pull --rebase` does; copes with an upstream
        // that was itself rebased.
        PullMode::Rebase => &["rebase", "--fork-point", "@{upstream}"],
    };
    match run_git(repo, args) {
        Ok(_) => Ok(()),
        Err(err) => Err(explain_pull_error(repo, mode, err)),
    }
}

/// Merge or rebase, following the user's git config like `git pull` would:
/// `branch.<name>.rebase` first, then `pull.rebase`. Unset means merge.
pub fn default_mode(repo: &Path) -> Result<PullMode, AppError> {
    let status = read_status(repo)?;
    let mut value = String::new();
    if let Some(name) = &status.branch.name {
        value = config_value(repo, &format!("branch.{name}.rebase"))?;
    }
    if value.is_empty() {
        value = config_value(repo, "pull.rebase")?;
    }
    Ok(if wants_rebase(&value) {
        PullMode::Rebase
    } else {
        PullMode::Merge
    })
}

// "" when the key is not set. (--default avoids exit code 1 for a missing key,
// which would look like a failure.)
fn config_value(repo: &Path, key: &str) -> Result<String, AppError> {
    let out = run_git(repo, &["config", "--default", "", "--get", key])?;
    Ok(out.trim().to_string())
}

// git accepts several spellings for "true"; "merges"/"interactive" also rebase.
fn wants_rebase(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "true" | "yes" | "on" | "1" | "merges" | "m" | "interactive" | "i"
    )
}

fn explain_pull_error(repo: &Path, mode: PullMode, err: AppError) -> AppError {
    // git prints the conflict report on stdout, which errors don't carry;
    // the status tells us reliably whether the merge/rebase stopped on conflicts.
    if let Ok(status) = read_status(repo) {
        let count = status.conflicted.len();
        if count > 0 {
            let next = match mode {
                PullMode::Rebase => "then Continue in the Rebasing bar (Abort undoes the pull)",
                _ => "then commit (Abort Merge undoes the pull)",
            };
            return AppError::new(
                ErrorKind::Conflict,
                format!(
                    "The pull stopped because {count} file(s) have conflicts. Double-click a file to \
                     resolve it, {next}."
                ),
            );
        }
    }
    if err.message.contains("Not possible to fast-forward") {
        return AppError::new(
            ErrorKind::InvalidInput,
            "The branch and its upstream have both changed; choose Merge or Rebase.".to_string(),
        );
    }
    explain_commit_error(err)
}

fn invalid(message: &str) -> AppError {
    AppError::new(ErrorKind::InvalidInput, message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_support::{clone_of, commit_file, git, server_and_clone};
    use std::fs;

    /// After this, `mine` is behind by one commit (theirs.txt) and, with
    /// `local_commit`, also ahead by one (mine.txt).
    fn diverged(local_commit: bool) -> (tempfile::TempDir, tempfile::TempDir) {
        let (server, mine) = server_and_clone();
        let theirs = clone_of(server.path());
        commit_file(theirs.path(), "theirs.txt");
        git(theirs.path(), &["push", "--quiet", "origin", "main"]);
        if local_commit {
            commit_file(mine.path(), "mine.txt");
        }
        git(mine.path(), &["fetch", "--quiet"]);
        (server, mine)
    }

    fn ahead_behind(dir: &Path) -> (u32, u32) {
        let branch = read_status(dir).unwrap().branch;
        (branch.ahead, branch.behind)
    }

    fn parents_of_head(dir: &Path) -> usize {
        // "%P": parent ids separated by spaces; a merge commit has two.
        git(dir, &["log", "-1", "--format=%P"])
            .split_whitespace()
            .count()
    }

    #[test]
    fn fast_forward_when_only_behind() {
        let (_server, mine) = diverged(false);
        assert_eq!(ahead_behind(mine.path()), (0, 1));

        pull(mine.path(), PullMode::FastForward).unwrap();

        assert_eq!(ahead_behind(mine.path()), (0, 0));
        assert!(mine.path().join("theirs.txt").exists());
    }

    #[test]
    fn fast_forward_refuses_diverged_branches() {
        let (_server, mine) = diverged(true);
        let err = pull(mine.path(), PullMode::FastForward).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
        assert_eq!(ahead_behind(mine.path()), (1, 1));
    }

    #[test]
    fn merge_creates_a_merge_commit() {
        let (_server, mine) = diverged(true);

        pull(mine.path(), PullMode::Merge).unwrap();

        assert_eq!(parents_of_head(mine.path()), 2);
        // My commit + the merge commit are now ahead; nothing is behind.
        assert_eq!(ahead_behind(mine.path()), (2, 0));
        assert!(mine.path().join("theirs.txt").exists());
    }

    #[test]
    fn rebase_puts_local_commits_on_top() {
        let (_server, mine) = diverged(true);

        pull(mine.path(), PullMode::Rebase).unwrap();

        assert_eq!(parents_of_head(mine.path()), 1);
        assert_eq!(ahead_behind(mine.path()), (1, 0));
        assert_eq!(
            git(mine.path(), &["log", "-1", "--format=%s"]).trim(),
            "mine.txt"
        );
        assert!(mine.path().join("theirs.txt").exists());
    }

    #[test]
    fn conflict_is_reported_as_conflict() {
        let (server, mine) = server_and_clone();
        let theirs = clone_of(server.path());
        fs::write(theirs.path().join("first.txt"), "theirs\n").unwrap();
        git(theirs.path(), &["commit", "--quiet", "-am", "theirs"]);
        git(theirs.path(), &["push", "--quiet", "origin", "main"]);
        fs::write(mine.path().join("first.txt"), "mine\n").unwrap();
        git(mine.path(), &["commit", "--quiet", "-am", "mine"]);
        git(mine.path(), &["fetch", "--quiet"]);

        for mode in [PullMode::Merge, PullMode::Rebase] {
            let err = pull(mine.path(), mode).unwrap_err();
            assert_eq!(err.kind, ErrorKind::Conflict, "{mode:?}: {}", err.message);
            assert_eq!(read_status(mine.path()).unwrap().conflicted.len(), 1);
            // Undo, so the next mode starts from the same state.
            let undo = if mode == PullMode::Merge {
                "merge"
            } else {
                "rebase"
            };
            git(mine.path(), &[undo, "--abort"]);
        }
    }

    #[test]
    fn needs_an_upstream() {
        let (_server, mine) = server_and_clone();
        git(mine.path(), &["checkout", "--quiet", "-b", "local-only"]);
        let err = pull(mine.path(), PullMode::Merge).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
    }

    #[test]
    fn default_mode_follows_git_config() {
        let (_server, mine) = server_and_clone();
        let root = mine.path();
        // The machine's global config may set pull.rebase; override it locally.
        git(root, &["config", "pull.rebase", "false"]);
        assert_eq!(default_mode(root).unwrap(), PullMode::Merge);
        git(root, &["config", "pull.rebase", "true"]);
        assert_eq!(default_mode(root).unwrap(), PullMode::Rebase);
        // A per-branch setting wins over pull.rebase.
        git(root, &["config", "branch.main.rebase", "false"]);
        assert_eq!(default_mode(root).unwrap(), PullMode::Merge);
    }

    #[test]
    fn reads_git_boolean_spellings() {
        for yes in ["true", "Yes", "on", "1", "merges", "interactive"] {
            assert!(wants_rebase(yes), "{yes}");
        }
        for no in ["", "false", "no", "off", "0"] {
            assert!(!wants_rebase(no), "{no}");
        }
    }
}
