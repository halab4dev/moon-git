use std::path::Path;

use tauri::{AppHandle, Manager, State};

use super::{emit_progress, run_blocking};
use crate::error::AppError;
use crate::git;
use crate::git::operations::Operations;
use crate::models::RepoEntry;
use crate::storage::repos::RepoStore;

/// Clones a repository and adds it to the list.
/// Progress arrives as "git-progress" events tagged with `op_id`;
/// `cancel_operation(op_id)` stops it.
#[tauri::command]
pub async fn clone_repo(
    app: AppHandle,
    op_id: String,
    url: String,
    parent_dir: String,
    folder_name: String,
    group_id: Option<String>,
) -> Result<RepoEntry, AppError> {
    // Cloning can take minutes; the UI thread stays free (and can call cancel).
    run_blocking(move || {
        let ops = app.state::<Operations>();
        let send_progress = |line: &str| emit_progress(&app, &op_id, line);

        let target = git::clone::clone_repo(
            &ops,
            &op_id,
            &url,
            Path::new(&parent_dir),
            &folder_name,
            send_progress,
        )?;
        let info = git::repo::check_repo(&target)?;
        app.state::<RepoStore>().add_repo(info, group_id)
    })
    .await
}

#[tauri::command]
pub fn cancel_operation(op_id: String, ops: State<'_, Operations>) -> Result<(), AppError> {
    ops.cancel(&op_id)
}

#[tauri::command]
pub fn suggest_folder_name(url: String) -> String {
    git::clone::folder_name_from_url(&url)
}
