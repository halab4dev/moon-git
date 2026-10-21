use tauri::State;

use super::run_blocking;
use crate::error::AppError;
use crate::git;
use crate::models::{HeadCommit, RepoStatus};
use crate::storage::repos::RepoStore;

/// Commits (or amends) and returns the new status, like stage/unstage.
#[tauri::command]
pub async fn commit_changes(
    repo_id: String,
    message: String,
    amend: bool,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        git::commit::commit(&repo, &message, amend)?;
        git::status::read_status(&repo)
    })
    .await
}

/// The last commit (message + whether it was pushed), or null before the first commit.
#[tauri::command]
pub async fn get_head_commit(
    repo_id: String,
    store: State<'_, RepoStore>,
) -> Result<Option<HeadCommit>, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || git::commit::head_commit(&repo)).await
}
