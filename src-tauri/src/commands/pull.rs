use tauri::State;

use super::run_blocking;
use crate::error::AppError;
use crate::git;
use crate::models::{PullMode, RepoStatus};
use crate::storage::repos::RepoStore;

/// Merges or rebases the fetched upstream into the current branch and
/// returns the new status. The frontend calls `fetch_repo` first.
#[tauri::command]
pub async fn pull_repo(
    repo_id: String,
    mode: PullMode,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        // On a conflict the error is returned, but the status changed too;
        // the frontend reloads it in that case.
        git::pull::pull(&repo, mode)?;
        git::status::read_status(&repo)
    })
    .await
}

/// Preselected choice for "merge or rebase?", from the user's git config.
#[tauri::command]
pub async fn get_pull_default(
    repo_id: String,
    store: State<'_, RepoStore>,
) -> Result<PullMode, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || git::pull::default_mode(&repo)).await
}
