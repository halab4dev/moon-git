use tauri::{AppHandle, Manager, State};

use super::{emit_progress, run_blocking};
use crate::error::AppError;
use crate::git;
use crate::git::operations::Operations;
use crate::storage::repos::RepoStore;

/// Pushes the current branch, with "git-progress" events tagged `op_id`.
/// `new_upstream`: remote name for a branch's first push, otherwise null.
#[tauri::command]
pub async fn push_repo(
    app: AppHandle,
    op_id: String,
    repo_id: String,
    new_upstream: Option<String>,
) -> Result<(), AppError> {
    let repo = app.state::<RepoStore>().repo_path(&repo_id)?;
    run_blocking(move || {
        let ops = app.state::<Operations>();
        git::push::push(&ops, &op_id, &repo, new_upstream.as_deref(), |line| {
            emit_progress(&app, &op_id, line)
        })
    })
    .await
}

#[tauri::command]
pub async fn list_remotes(
    repo_id: String,
    store: State<'_, RepoStore>,
) -> Result<Vec<String>, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || git::push::list_remotes(&repo)).await
}
