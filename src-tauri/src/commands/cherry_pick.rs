use tauri::State;

use super::run_blocking;
use crate::error::AppError;
use crate::git;
use crate::models::{CherryPickAction, CherryPickMode, RepoStatus};
use crate::storage::repos::RepoStore;

/// Applies the changes of `commit` (an id) to the current branch. Returns the
/// new status; on conflicts, fails with kind "conflict".
#[tauri::command]
pub async fn cherry_pick_commit(
    repo_id: String,
    commit: String,
    mode: CherryPickMode,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        git::cherry_pick::cherry_pick(&repo, &commit, mode)?;
        git::status::read_status(&repo)
    })
    .await
}

/// Continue or abort the cherry-pick that waits (the UI asks before abort).
#[tauri::command]
pub async fn run_cherry_pick_action(
    repo_id: String,
    action: CherryPickAction,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        git::cherry_pick::run(&repo, action)?;
        git::status::read_status(&repo)
    })
    .await
}
