// Creating commits, and reading the last one back for "amend".

use std::path::Path;

use crate::error::{AppError, ErrorKind};
use crate::git::rebase::rebase_in_progress;
use crate::git::repo::has_commits;
use crate::git::runner::{run_git, run_git_with_input};
use crate::git::status::read_status;
use crate::models::HeadCommit;

/// Commits the staged changes with `message`.
///
/// With `amend`, the last commit is replaced instead: its changes plus
/// whatever is staged now, under the new message. Nothing staged is fine
/// then (only the message changes).
///
/// During a merge, the commit becomes the merge commit (two parents). With
/// `simple`, it becomes an ordinary commit instead (one parent, like
/// `merge --squash`): the same changes, but the history does not record
/// that the other branch was merged. SmartGit offers the same choice.
pub fn commit(repo: &Path, message: &str, amend: bool, simple: bool) -> Result<(), AppError> {
    if message.trim().is_empty() {
        return Err(invalid("Enter a commit message."));
    }
    if amend && !has_commits(repo) {
        return Err(invalid("There is no commit to amend yet."));
    }

    // Checked here because git's own answer is unhelpful: "nothing to commit"
    // goes to stdout with an empty stderr.
    let status = read_status(repo)?;
    if !status.conflicted.is_empty() {
        return Err(invalid(
            "Some files still have conflicts. Resolve them before committing.",
        ));
    }
    // Halfway through a rebase, HEAD is detached on a replayed commit; a
    // commit made now would not end up where the user expects.
    if rebase_in_progress(repo) {
        return Err(invalid(
            "A rebase is in progress. Use Continue (or Abort) in the Rebasing bar instead of \
             committing.",
        ));
    }
    if amend && status.merge.is_some() {
        return Err(invalid(
            "A merge is in progress. Commit or abort it before amending.",
        ));
    }
    if !amend && status.staged.is_empty() {
        return Err(invalid(
            "Nothing is staged. Stage the files you want to commit first.",
        ));
    }

    // -F -: read the message from stdin, so it can never be taken for an option.
    // --cleanup=whitespace: only trim blank lines/spaces; keep lines starting
    // with '#' (e.g. "#123 fix login"), which the default for an editor would drop.
    if simple && status.merge.is_some() {
        // git makes a merge commit whenever MERGE_HEAD exists; removing it
        // turns this into an ordinary commit. git removes the other merge
        // files (MERGE_MSG, MERGE_MODE) itself after committing.
        run_git(repo, &["update-ref", "-d", "MERGE_HEAD"])?;
    }

    let mut args = vec!["commit", "--quiet", "--cleanup=whitespace", "-F", "-"];
    if amend {
        args.push("--amend");
    }
    run_git_with_input(repo, &args, message).map_err(explain_commit_error)?;
    Ok(())
}

/// The last commit's message, or None before the first commit.
pub fn head_commit(repo: &Path) -> Result<Option<HeadCommit>, AppError> {
    if !has_commits(repo) {
        return Ok(None);
    }
    // %B: the raw message (subject + body). --no-show-signature: no GPG output
    // even if the user enabled log.showSignature.
    let message = run_git(
        repo,
        &["log", "-1", "--no-show-signature", "--format=%B", "HEAD"],
    )?;
    // Lists the remote-tracking branches that already contain HEAD.
    let remotes = run_git(repo, &["branch", "--remotes", "--contains", "HEAD"])?;

    Ok(Some(HeadCommit {
        message: message.trim_end().to_string(),
        pushed: !remotes.trim().is_empty(),
    }))
}

fn invalid(message: &str) -> AppError {
    AppError::new(ErrorKind::InvalidInput, message.to_string())
}

// git's "Please tell me who you are" text suggests terminal commands;
// say what is wrong in a way that fits the app.
// Also used by pull: merges and rebases create commits too.
pub(crate) fn explain_commit_error(err: AppError) -> AppError {
    let identity_problem = err.message.contains("Please tell me who you are")
        || err.message.contains("Author identity unknown")
        || err.message.contains("empty ident name");
    if err.kind == ErrorKind::GitFailed && identity_problem {
        return AppError::new(
            ErrorKind::IdentityUnknown,
            "git does not know your name and email for commits. Set them with \
             `git config --global user.name \"Your Name\"` and \
             `git config --global user.email you@example.com` \
             (a per-repository setting will come in a later step)."
                .to_string(),
        );
    }
    err
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn git(dir: &Path, args: &[&str]) -> String {
        run_git(dir, args).unwrap()
    }

    // The identity is set in the repo itself, so commit() works on machines
    // without a global user.name/user.email.
    fn new_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        git(dir.path(), &["init", "--quiet"]);
        git(dir.path(), &["config", "user.name", "Test"]);
        git(dir.path(), &["config", "user.email", "test@example.com"]);
        dir
    }

    fn write_and_stage(dir: &Path, name: &str, text: &str) {
        fs::write(dir.join(name), text).unwrap();
        git(dir, &["add", "--", name]);
    }

    fn commit_count(dir: &Path) -> usize {
        git(dir, &["rev-list", "--count", "HEAD"])
            .trim()
            .parse()
            .unwrap()
    }

    fn files_in_head(dir: &Path) -> String {
        git(dir, &["ls-tree", "--name-only", "HEAD"])
    }

    #[test]
    fn commits_staged_files_only() {
        let dir = new_repo();
        let root = dir.path();
        write_and_stage(root, "a.txt", "a\n");
        fs::write(root.join("not-staged.txt"), "x\n").unwrap();

        commit(root, "First commit", false, false).unwrap();

        assert_eq!(commit_count(root), 1);
        assert_eq!(files_in_head(root), "a.txt\n");
        let head = head_commit(root).unwrap().unwrap();
        assert_eq!(head.message, "First commit");
        assert!(!head.pushed);
    }

    #[test]
    fn keeps_multi_line_message_and_hash_lines() {
        let dir = new_repo();
        let root = dir.path();
        write_and_stage(root, "a.txt", "a\n");
        // A message that looks like an option must not be read as one.
        let message = "--amend is not an option here\n\n#123 body line\n- item";

        commit(root, message, false, false).unwrap();

        assert_eq!(head_commit(root).unwrap().unwrap().message, message);
    }

    #[test]
    fn amend_replaces_last_commit() {
        let dir = new_repo();
        let root = dir.path();
        write_and_stage(root, "a.txt", "a\n");
        commit(root, "Frist", false, false).unwrap();
        write_and_stage(root, "b.txt", "b\n");

        commit(root, "First", true, false).unwrap();

        assert_eq!(commit_count(root), 1);
        assert_eq!(files_in_head(root), "a.txt\nb.txt\n");
        assert_eq!(head_commit(root).unwrap().unwrap().message, "First");
    }

    #[test]
    fn amend_without_staged_changes_only_rewords() {
        let dir = new_repo();
        let root = dir.path();
        write_and_stage(root, "a.txt", "a\n");
        commit(root, "old message", false, false).unwrap();

        commit(root, "new message", true, false).unwrap();

        assert_eq!(commit_count(root), 1);
        assert_eq!(head_commit(root).unwrap().unwrap().message, "new message");
    }

    #[test]
    fn rejects_empty_message_nothing_staged_and_amend_without_commits() {
        let dir = new_repo();
        let root = dir.path();
        write_and_stage(root, "a.txt", "a\n");

        assert_eq!(
            commit(root, "  \n ", false, false).unwrap_err().kind,
            ErrorKind::InvalidInput
        );
        assert_eq!(
            commit(root, "msg", true, false).unwrap_err().kind,
            ErrorKind::InvalidInput
        );

        commit(root, "msg", false, false).unwrap();
        let err = commit(root, "again", false, false).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
        assert!(err.message.contains("Nothing is staged"), "{}", err.message);
    }

    #[test]
    fn rejects_commit_while_conflicted() {
        let dir = new_repo();
        let root = dir.path();
        write_and_stage(root, "a.txt", "base\n");
        commit(root, "base", false, false).unwrap();
        git(root, &["checkout", "--quiet", "-b", "other"]);
        write_and_stage(root, "a.txt", "other\n");
        commit(root, "other", false, false).unwrap();
        git(root, &["checkout", "--quiet", "-"]);
        write_and_stage(root, "a.txt", "main\n");
        commit(root, "main", false, false).unwrap();
        // Exits non-zero because of the conflict; that is the point.
        let _ = run_git(root, &["merge", "--quiet", "other"]);

        let err = commit(root, "merge", false, false).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
        assert!(err.message.contains("conflicts"), "{}", err.message);
    }

    #[test]
    fn head_commit_is_none_before_first_commit() {
        let dir = new_repo();
        assert_eq!(head_commit(dir.path()).unwrap(), None);
    }

    #[test]
    fn head_commit_knows_when_it_was_pushed() {
        let remote = TempDir::new().unwrap();
        git(remote.path(), &["init", "--quiet", "--bare"]);
        let dir = new_repo();
        let root = dir.path();
        let remote_path = remote.path().to_string_lossy().to_string();
        git(root, &["remote", "add", "origin", &remote_path]);
        write_and_stage(root, "a.txt", "a\n");
        commit(root, "pushed", false, false).unwrap();
        git(root, &["push", "--quiet", "origin", "HEAD:refs/heads/main"]);
        git(root, &["fetch", "--quiet", "origin"]);

        assert!(head_commit(root).unwrap().unwrap().pushed);

        write_and_stage(root, "b.txt", "b\n");
        commit(root, "local only", false, false).unwrap();
        assert!(!head_commit(root).unwrap().unwrap().pushed);
    }

    #[test]
    fn explains_missing_identity() {
        let raw = AppError::new(
            ErrorKind::GitFailed,
            "Author identity unknown\n\n*** Please tell me who you are.".to_string(),
        );
        assert_eq!(explain_commit_error(raw).kind, ErrorKind::IdentityUnknown);

        let other = AppError::new(ErrorKind::GitFailed, "hook failed".to_string());
        assert_eq!(explain_commit_error(other).kind, ErrorKind::GitFailed);
    }

    // main and "other" both have a commit; "other" merged into main without
    // committing (like "Merge to Working Tree").
    fn merge_waiting_for_commit() -> TempDir {
        let dir = new_repo();
        let root = dir.path();
        git(root, &["symbolic-ref", "HEAD", "refs/heads/main"]);
        write_and_stage(root, "base.txt", "base\n");
        commit(root, "base", false, false).unwrap();
        git(root, &["switch", "--quiet", "--create", "other"]);
        write_and_stage(root, "other.txt", "other\n");
        commit(root, "other", false, false).unwrap();
        git(root, &["switch", "--quiet", "main"]);
        write_and_stage(root, "main.txt", "main\n");
        commit(root, "main", false, false).unwrap();
        git(
            root,
            &["merge", "--quiet", "--no-ff", "--no-commit", "other"],
        );
        dir
    }

    fn parent_count(dir: &Path) -> usize {
        git(dir, &["log", "-1", "--format=%P"])
            .split_whitespace()
            .count()
    }

    #[test]
    fn commit_during_a_merge_is_the_merge_commit() {
        let dir = merge_waiting_for_commit();
        let root = dir.path();
        commit(root, "Merge other", false, false).unwrap();
        assert_eq!(parent_count(root), 2);
        assert_eq!(read_status(root).unwrap().merge, None);
    }

    #[test]
    fn simple_commit_during_a_merge_has_one_parent() {
        let dir = merge_waiting_for_commit();
        let root = dir.path();
        commit(root, "Bring in other", false, true).unwrap();
        assert_eq!(parent_count(root), 1);
        assert_eq!(files_in_head(root), "base.txt\nmain.txt\nother.txt\n");
        assert_eq!(read_status(root).unwrap().merge, None);
    }

    #[test]
    fn no_amend_during_a_merge() {
        let dir = merge_waiting_for_commit();
        let root = dir.path();
        let err = commit(root, "x", true, false).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
        // Still merging.
        assert!(read_status(root).unwrap().merge.is_some());
    }
}
