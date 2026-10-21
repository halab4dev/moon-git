use tauri::State;

use super::run_blocking;
use crate::error::AppError;
use crate::git;
use crate::models::{LogPage, RefInfo};
use crate::storage::repos::RepoStore;

/// One page of the history: `limit` commits after the first `skip`.
/// `refs`: full ref names to show; null = all branches, remotes and tags.
/// `filter`: only commits matching this text (hide mode), with rewritten parents.
#[tauri::command]
pub async fn get_log(
    repo_id: String,
    skip: usize,
    limit: usize,
    refs: Option<Vec<String>>,
    filter: Option<String>,
    store: State<'_, RepoStore>,
) -> Result<LogPage, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        let refs = refs.as_deref();
        match filter.as_deref().map(str::trim) {
            Some(text) if !text.is_empty() => {
                git::search::read_filtered_log(&repo, skip, limit, refs, text)
            }
            _ => git::log::read_log(&repo, skip, limit, refs),
        }
    })
    .await
}

/// Ids of all commits matching `text` (highlight mode), newest first.
#[tauri::command]
pub async fn find_commits(
    repo_id: String,
    refs: Option<Vec<String>>,
    text: String,
    store: State<'_, RepoStore>,
) -> Result<Vec<String>, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || git::search::find_matches(&repo, refs.as_deref(), &text)).await
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
