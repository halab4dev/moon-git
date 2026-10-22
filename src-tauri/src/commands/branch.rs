use tauri::State;

use super::run_blocking;
use crate::error::AppError;
use crate::git;
use crate::models::RepoStatus;
use crate::storage::repos::RepoStore;

// Both return the new status (the current branch changed).

/// `full_name`: "refs/heads/x" or "refs/remotes/origin/x" (see list_refs).
#[tauri::command]
pub async fn checkout_branch(
    repo_id: String,
    full_name: String,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        git::branch::checkout(&repo, &full_name)?;
        git::status::read_status(&repo)
    })
    .await
}

/// `start`: commit id or full ref name; null = current HEAD.
#[tauri::command]
pub async fn create_branch(
    repo_id: String,
    name: String,
    start: Option<String>,
    check_out: bool,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        git::branch::create(&repo, &name, start.as_deref(), check_out)?;
        git::status::read_status(&repo)
    })
    .await
}
