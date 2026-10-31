use tauri::State;

use super::{apply_to_repos, run_blocking};
use crate::error::AppError;
use crate::error::ErrorKind;
use crate::git;
use crate::models::{GroupAccess, GroupApplyResult, RemoteAccess, RepoEntry};
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

/// Writes login settings into the repositories of a group and its subgroups
/// (step 15b): only into parts a repository hasn't set itself, or into all
/// of them (`all`). The group remembers the choice for its dialog.
#[tauri::command]
pub async fn apply_group_access(
    group_id: String,
    ssh_key: Option<String>,
    system_credentials: bool,
    all: bool,
    store: State<'_, RepoStore>,
) -> Result<GroupApplyResult, AppError> {
    let ssh_key = match ssh_key {
        Some(key) => Some(git::access::checked_key(&key)?),
        None => None,
    };
    if store.group(&group_id)?.is_none() {
        return Err(AppError::new(
            ErrorKind::NotFound,
            "This group no longer exists.".to_string(),
        ));
    }
    let repos = store.repos_in_group(&group_id)?;
    let total = repos.len() as u32;
    let (changed, failed) = write_access(repos, ssh_key.clone(), system_credentials, all).await?;
    // Remembered for the group's dialog.
    let list = store.set_group_access(
        &group_id,
        GroupAccess {
            ssh_key,
            system_credentials,
        },
    )?;
    Ok(GroupApplyResult {
        list,
        total,
        changed,
        failed,
    })
}

/// The same for repositories picked in the sidebar (step 35c).
#[tauri::command]
pub async fn apply_access_to_repos(
    repo_ids: Vec<String>,
    ssh_key: Option<String>,
    system_credentials: bool,
    all: bool,
    store: State<'_, RepoStore>,
) -> Result<GroupApplyResult, AppError> {
    let ssh_key = match ssh_key {
        Some(key) => Some(git::access::checked_key(&key)?),
        None => None,
    };
    let repos = store.repos_by_ids(&repo_ids)?;
    let total = repos.len() as u32;
    let (changed, failed) = write_access(repos, ssh_key, system_credentials, all).await?;
    Ok(GroupApplyResult {
        list: store.list()?,
        total,
        changed,
        failed,
    })
}

async fn write_access(
    repos: Vec<RepoEntry>,
    ssh_key: Option<String>,
    system_credentials: bool,
    all: bool,
) -> Result<(u32, Vec<String>), AppError> {
    apply_to_repos(repos, move |path| {
        git::access::apply_group_access(path, ssh_key.as_deref(), system_credentials, all)
    })
    .await
}
