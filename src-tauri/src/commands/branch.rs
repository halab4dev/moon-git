use tauri::{AppHandle, Manager, State};

use super::{emit_progress, run_blocking};
use crate::error::AppError;
use crate::git;
use crate::git::operations::Operations;
use crate::models::{BranchList, RepoStatus};
use crate::storage::repos::RepoStore;

// All return the new status (the branch shown in the header may have changed).

/// `full_name`: "refs/heads/x" or "refs/remotes/origin/x" (see list_refs).
#[tauri::command]
pub async fn checkout_branch(
    repo_id: String,
    full_name: String,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        git::branch::checkout(&repo, &full_name)?;
        git::status::read_status(&repo)
    })
    .await
}

/// `start`: commit id or full ref name; null = current HEAD.
#[tauri::command]
pub async fn create_branch(
    repo_id: String,
    name: String,
    start: Option<String>,
    check_out: bool,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        git::branch::create(&repo, &name, start.as_deref(), check_out)?;
        git::status::read_status(&repo)
    })
    .await
}

/// `new_name` for local branch `full_name` ("refs/heads/x").
#[tauri::command]
pub async fn rename_branch(
    repo_id: String,
    full_name: String,
    new_name: String,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        git::branch::rename(&repo, &full_name, &new_name)?;
        git::status::read_status(&repo)
    })
    .await
}

/// Local branch. Fails with kind "notMerged" unless `force` (asked separately in the UI).
#[tauri::command]
pub async fn delete_branch(
    repo_id: String,
    full_name: String,
    force: bool,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        git::branch::delete_local(&repo, &full_name, force)?;
        git::status::read_status(&repo)
    })
    .await
}

/// Branch on the server, given as "refs/remotes/<remote>/<branch>". Talks to
/// the server, so it runs like fetch/push: "git-progress" events, cancellable.
#[tauri::command]
pub async fn delete_remote_branch(
    app: AppHandle,
    op_id: String,
    repo_id: String,
    full_name: String,
) -> Result<RepoStatus, AppError> {
    let repo = app.state::<RepoStore>().repo_path(&repo_id)?;
    run_blocking(move || {
        let ops = app.state::<Operations>();
        git::branch::delete_remote(&ops, &op_id, &repo, &full_name, |line| {
            emit_progress(&app, &op_id, line)
        })?;
        git::status::read_status(&repo)
    })
    .await
}

/// `upstream`: "refs/remotes/origin/x", or null to stop tracking.
#[tauri::command]
pub async fn set_branch_upstream(
    repo_id: String,
    full_name: String,
    upstream: Option<String>,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        git::branch::set_upstream(&repo, &full_name, upstream.as_deref())?;
        git::status::read_status(&repo)
    })
    .await
}

/// Local branches and remote branches grouped by remote, for the sidebar.
#[tauri::command]
pub async fn list_branches(
    repo_id: String,
    store: State<'_, RepoStore>,
) -> Result<BranchList, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || git::branches::list_branches(&repo)).await
}

/// Adds remote `name` with `url` (step 26). Nothing is fetched yet.
#[tauri::command]
pub async fn add_remote(
    repo_id: String,
    name: String,
    url: String,
    store: State<'_, RepoStore>,
) -> Result<(), AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || git::remote::add(&repo, &name, &url)).await
}

/// Changes the URL of remote `name`.
#[tauri::command]
pub async fn set_remote_url(
    repo_id: String,
    name: String,
    url: String,
    store: State<'_, RepoStore>,
) -> Result<(), AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || git::remote::set_url(&repo, &name, &url)).await
}
