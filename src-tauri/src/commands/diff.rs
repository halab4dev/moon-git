use tauri::State;

use super::run_blocking;
use crate::error::AppError;
use crate::git;
use crate::models::{DiffMode, FileDiff};
use crate::storage::repos::RepoStore;

#[tauri::command]
pub async fn get_file_diff(
    repo_id: String,
    path: String,
    orig_path: Option<String>,
    mode: DiffMode,
    store: State<'_, RepoStore>,
) -> Result<FileDiff, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || git::diff::read_diff(&repo, &path, orig_path.as_deref(), mode)).await
}
