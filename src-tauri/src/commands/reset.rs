use tauri::State;

use super::run_blocking;
use crate::error::AppError;
use crate::git;
use crate::models::{RepoStatus, ResetMode, ResetPreview};
use crate::storage::repos::RepoStore;

/// `target`: full ref name or a commit id.
#[tauri::command]
pub async fn get_reset_preview(
    repo_id: String,
    target: String,
    store: State<'_, RepoStore>,
) -> Result<ResetPreview, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || git::reset::preview(&repo, &target)).await
}

/// Moves the current branch to `target` (the UI confirms first, twice for
/// hard). Returns the new status.
#[tauri::command]
pub async fn reset_current(
    repo_id: String,
    target: String,
    mode: ResetMode,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        git::reset::reset(&repo, &target, mode)?;
        git::status::read_status(&repo)
    })
    .await
}
