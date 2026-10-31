use tauri::State;

use super::run_blocking;
use crate::error::AppError;
use crate::git;
use crate::models::RemoteAccess;
use crate::storage::repos::RepoStore;

// SSH key and HTTPS login of one repository (step 15a), in its .git/config.

#[tauri::command]
pub async fn get_remote_access(
    repo_id: String,
    store: State<'_, RepoStore>,
) -> Result<RemoteAccess, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || git::access::read(&repo)).await
}

/// `ssh_key` null: this computer's SSH settings. `system_credentials`: HTTPS
/// logins from this computer's credential helpers instead of MoonGit's accounts.
#[tauri::command]
pub async fn set_remote_access(
    repo_id: String,
    ssh_key: Option<String>,
    system_credentials: bool,
    store: State<'_, RepoStore>,
) -> Result<RemoteAccess, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || git::access::set(&repo, ssh_key.as_deref(), system_credentials)).await
}
