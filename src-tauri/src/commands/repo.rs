use std::path::Path;

use tauri::State;

use crate::error::AppError;
use crate::git;
use crate::models::RepoEntry;
use crate::storage::repos::RepoStore;

#[tauri::command]
pub fn list_repos(store: State<'_, RepoStore>) -> Result<Vec<RepoEntry>, AppError> {
    store.list()
}

/// Validates `path` with git, then saves the repository root (not the subfolder).
#[tauri::command]
pub fn add_repo(path: String, store: State<'_, RepoStore>) -> Result<RepoEntry, AppError> {
    let info = git::repo::check_repo(Path::new(&path))?;
    store.add(info)
}

#[tauri::command]
pub fn remove_repo(id: String, store: State<'_, RepoStore>) -> Result<(), AppError> {
    store.remove(&id)
}
