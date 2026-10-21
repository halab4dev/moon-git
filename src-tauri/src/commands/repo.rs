use std::path::Path;

use tauri::State;

use crate::error::AppError;
use crate::git;
use crate::models::{RepoEntry, RepoList};
use crate::storage::repos::RepoStore;

// Commands that change the list return the whole updated list, so the
// frontend never has to repeat the rules (e.g. what deleting a group does).

#[tauri::command]
pub fn list_repos(store: State<'_, RepoStore>) -> Result<RepoList, AppError> {
    store.list()
}

/// Validates `path` with git, then saves the repository root (not the subfolder).
#[tauri::command]
pub fn add_repo(
    path: String,
    group_id: Option<String>,
    store: State<'_, RepoStore>,
) -> Result<RepoEntry, AppError> {
    let info = git::repo::check_repo(Path::new(&path))?;
    store.add_repo(info, group_id)
}

#[tauri::command]
pub fn remove_repo(id: String, store: State<'_, RepoStore>) -> Result<RepoList, AppError> {
    store.remove_repo(&id)
}

#[tauri::command]
pub fn move_repo(
    repo_id: String,
    group_id: Option<String>,
    store: State<'_, RepoStore>,
) -> Result<RepoList, AppError> {
    store.move_repo(&repo_id, group_id)
}
