use tauri::State;

use super::run_blocking;
use crate::error::AppError;
use crate::git;
use crate::models::RepoStatus;
use crate::storage::repos::RepoStore;

// All return the new status, so the UI can redraw without a second call.

#[tauri::command]
pub async fn stage_files(
    repo_id: String,
    paths: Vec<String>,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        git::stage::stage(&repo, &paths)?;
        git::status::read_status(&repo)
    })
    .await
}

#[tauri::command]
pub async fn unstage_files(
    repo_id: String,
    paths: Vec<String>,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        git::stage::unstage(&repo, &paths)?;
        git::status::read_status(&repo)
    })
    .await
}

/// Discards the changes of `paths` (the UI asks first): back to HEAD, or to
/// the Trash for files HEAD doesn't have. Returns the new status.
#[tauri::command]
pub async fn discard_changes(
    repo_id: String,
    paths: Vec<String>,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        git::discard::discard(&repo, &paths)?;
        git::status::read_status(&repo)
    })
    .await
}
