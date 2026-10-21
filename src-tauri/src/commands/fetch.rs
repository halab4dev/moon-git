use tauri::{AppHandle, Manager};

use super::{emit_progress, run_blocking};
use crate::error::AppError;
use crate::git;
use crate::git::operations::Operations;
use crate::storage::repos::RepoStore;

/// Fetches from all remotes. Progress arrives as "git-progress" events
/// tagged with `op_id`; `cancel_operation(op_id)` stops it.
/// Returns nothing: the frontend reloads the status (ahead/behind) afterwards.
#[tauri::command]
pub async fn fetch_repo(app: AppHandle, op_id: String, repo_id: String) -> Result<(), AppError> {
    let repo = app.state::<RepoStore>().repo_path(&repo_id)?;
    run_blocking(move || {
        let ops = app.state::<Operations>();
        git::fetch::fetch(&ops, &op_id, &repo, |line| {
            emit_progress(&app, &op_id, line)
        })
    })
    .await
}
