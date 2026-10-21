use tauri::State;

use crate::error::AppError;
use crate::models::RepoList;
use crate::storage::repos::RepoStore;

#[tauri::command]
pub fn create_group(
    name: String,
    parent_id: Option<String>,
    store: State<'_, RepoStore>,
) -> Result<RepoList, AppError> {
    store.create_group(&name, parent_id)
}

#[tauri::command]
pub fn rename_group(
    id: String,
    name: String,
    store: State<'_, RepoStore>,
) -> Result<RepoList, AppError> {
    store.rename_group(&id, &name)
}

/// Repos and subgroups inside move up one level; nothing on disk is touched.
#[tauri::command]
pub fn delete_group(id: String, store: State<'_, RepoStore>) -> Result<RepoList, AppError> {
    store.delete_group(&id)
}
