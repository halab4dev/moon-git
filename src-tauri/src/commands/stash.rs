use tauri::State;

use super::run_blocking;
use crate::error::AppError;
use crate::git;
use crate::models::{RepoStatus, StashEntry};
use crate::storage::repos::RepoStore;

/// Returns the new status: the stashed changes are gone from the working tree.
#[tauri::command]
pub async fn create_stash(
    repo_id: String,
    message: String,
    include_untracked: bool,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        git::stash::create(&repo, &message, include_untracked)?;
        git::status::read_status(&repo)
    })
    .await
}

#[tauri::command]
pub async fn list_stashes(
    repo_id: String,
    store: State<'_, RepoStore>,
) -> Result<Vec<StashEntry>, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || git::stash::list(&repo)).await
}
