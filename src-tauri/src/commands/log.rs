use tauri::State;

use super::run_blocking;
use crate::error::AppError;
use crate::git;
use crate::models::LogPage;
use crate::storage::repos::RepoStore;

/// One page of the history: `limit` commits after the first `skip`.
#[tauri::command]
pub async fn get_log(
    repo_id: String,
    skip: usize,
    limit: usize,
    store: State<'_, RepoStore>,
) -> Result<LogPage, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || git::log::read_log(&repo, skip, limit)).await
}
