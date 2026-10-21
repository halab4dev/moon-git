use tauri::State;

use super::run_blocking;
use crate::error::AppError;
use crate::git;
use crate::models::RepoStatus;
use crate::storage::repos::RepoStore;

/// Changed files of a saved repository. Takes the repo id (not a path) so the
/// frontend can only run git in folders the user added.
#[tauri::command]
pub async fn get_status(
    repo_id: String,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let path = store.repo_path(&repo_id)?;
    // On a big repository status can take a while; keep the UI responsive.
    run_blocking(move || git::status::read_status(&path)).await
}
