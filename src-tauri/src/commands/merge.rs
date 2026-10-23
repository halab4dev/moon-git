use tauri::State;

use super::run_blocking;
use crate::error::AppError;
use crate::git;
use crate::models::{MergeMode, MergePreview, RepoStatus};
use crate::storage::repos::RepoStore;

/// `source`: full ref name ("refs/heads/x", "refs/remotes/origin/x",
/// "refs/tags/v1") or a commit id.
#[tauri::command]
pub async fn get_merge_preview(
    repo_id: String,
    source: String,
    store: State<'_, RepoStore>,
) -> Result<MergePreview, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || git::merge::preview(&repo, &source)).await
}

/// Merges `source` into the current branch. Returns the new status; on
/// conflicts, fails with kind "conflict" (the files are left for the user).
#[tauri::command]
pub async fn merge_into_current(
    repo_id: String,
    source: String,
    mode: MergeMode,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        git::merge::merge(&repo, &source, mode)?;
        git::status::read_status(&repo)
    })
    .await
}
