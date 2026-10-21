use std::path::Path;

use tauri::{AppHandle, Emitter, Manager, State};

use super::run_blocking;
use crate::error::AppError;
use crate::git;
use crate::git::operations::{Operations, parse_percent};
use crate::models::{GitProgress, RepoEntry};
use crate::storage::repos::RepoStore;

/// Event name the frontend listens to for progress of any long git command.
pub const PROGRESS_EVENT: &str = "git-progress";

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
        let send_progress = |line: &str| {
            let (phase, percent) = match parse_percent(line) {
                Some((phase, percent)) => (Some(phase), Some(percent)),
                None => (None, None),
            };
            let event = GitProgress {
                op_id: op_id.clone(),
                line: line.to_string(),
                phase,
                percent,
            };
            if let Err(err) = app.emit(PROGRESS_EVENT, event) {
                eprintln!("Could not send progress: {err}");
            }
        };

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
