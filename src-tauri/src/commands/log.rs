use tauri::State;

use super::run_blocking;
use crate::error::AppError;
use crate::git;
use crate::models::{LogPage, RefInfo};
use crate::storage::repos::RepoStore;

/// One page of the history: `limit` commits after the first `skip`.
/// `refs`: full ref names to show; null = all branches, remotes and tags.
#[tauri::command]
pub async fn get_log(
    repo_id: String,
    skip: usize,
    limit: usize,
    refs: Option<Vec<String>>,
    store: State<'_, RepoStore>,
) -> Result<LogPage, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || git::log::read_log(&repo, skip, limit, refs.as_deref())).await
}

/// Branches, remote branches and tags, for choosing what the history shows.
#[tauri::command]
pub async fn list_refs(
    repo_id: String,
    store: State<'_, RepoStore>,
) -> Result<Vec<RefInfo>, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || git::refs::list_refs(&repo)).await
}
