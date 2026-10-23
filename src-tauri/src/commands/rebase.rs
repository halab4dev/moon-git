use tauri::State;

use super::run_blocking;
use crate::error::AppError;
use crate::git;
use crate::models::{RebasePreview, RepoStatus};
use crate::storage::repos::RepoStore;

/// `onto`: full ref name ("refs/heads/x", "refs/remotes/origin/x",
/// "refs/tags/v1") or a commit id.
#[tauri::command]
pub async fn get_rebase_preview(
    repo_id: String,
    onto: String,
    store: State<'_, RepoStore>,
) -> Result<RebasePreview, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || git::rebase::preview(&repo, &onto)).await
}

/// Rebases the current branch onto `onto`. Returns the new status; on
/// conflicts, fails with kind "conflict" (the rebase waits halfway).
#[tauri::command]
pub async fn rebase_current(
    repo_id: String,
    onto: String,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        git::rebase::rebase(&repo, &onto)?;
        git::status::read_status(&repo)
    })
    .await
}
