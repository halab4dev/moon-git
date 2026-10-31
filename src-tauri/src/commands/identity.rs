use std::path::Path;

use tauri::State;

use super::run_blocking;
use crate::error::{AppError, ErrorKind};
use crate::git;
use crate::git::identity::clean_identity;
use crate::models::{GroupIdentityResult, Identity};
use crate::storage::repos::RepoStore;

// Commit identities (step 14): of one repository (its .git/config), and of a
// group (step 14b), which is written into the .git/config of its
// repositories, so git in a terminal uses it too.

#[tauri::command]
pub async fn get_identity(
    repo_id: String,
    store: State<'_, RepoStore>,
) -> Result<Identity, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || git::identity::read(&repo)).await
}

/// `name` and `email` both null: remove the repository's own identity (the
/// computer-wide one applies again).
#[tauri::command]
pub async fn set_identity(
    repo_id: String,
    name: Option<String>,
    email: Option<String>,
    store: State<'_, RepoStore>,
) -> Result<Identity, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || git::identity::set(&repo, both(&name, &email))).await
}

/// The computer-wide identity (~/.gitconfig), for the group dialog.
#[tauri::command]
pub async fn get_computer_identity() -> Result<Identity, AppError> {
    run_blocking(git::identity::read_computer).await
}

/// Sets a group's identity (step 14b) and writes it into the .git/config of
/// the repositories of the group and its subgroups. `all`: also over the
/// identity a repository already has; otherwise only repositories without
/// one get it. A repository that can't be changed (folder gone…) doesn't stop
/// the others: it is reported in `failed`.
#[tauri::command]
pub async fn apply_group_identity(
    group_id: String,
    name: String,
    email: String,
    all: bool,
    store: State<'_, RepoStore>,
) -> Result<GroupIdentityResult, AppError> {
    let (name, email) = clean_identity(&name, &email)?;
    if store.group(&group_id)?.is_none() {
        return Err(AppError::new(
            ErrorKind::NotFound,
            "This group no longer exists.".to_string(),
        ));
    }
    let repos = store.repos_in_group(&group_id)?;
    let total = repos.len() as u32;

    let (written_name, written_email) = (name.clone(), email.clone());
    let (changed, failed) = run_blocking(move || {
        let mut changed = 0;
        let mut failed = Vec::new();
        for repo in repos {
            let path = Path::new(&repo.path);
            match git::identity::apply_group_identity(path, &written_name, &written_email, all) {
                Ok(true) => changed += 1,
                Ok(false) => {}
                Err(err) => failed.push(format!("{}: {}", repo.name, err.message)),
            }
        }
        Ok((changed, failed))
    })
    .await?;

    let list = store.set_group_identity(&group_id, Some((&name, &email)))?;
    Ok(GroupIdentityResult {
        list,
        total,
        changed,
        failed,
    })
}

fn both<'a>(name: &'a Option<String>, email: &'a Option<String>) -> Option<(&'a str, &'a str)> {
    match (name, email) {
        (Some(name), Some(email)) => Some((name.as_str(), email.as_str())),
        _ => None,
    }
}
