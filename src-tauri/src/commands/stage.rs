use tauri::State;

use super::run_blocking;
use crate::error::AppError;
use crate::git;
use crate::models::{ConflictChoice, ConflictVersions, RepoStatus};
use crate::storage::repos::RepoStore;

// All return the new status, so the UI can redraw without a second call.

#[tauri::command]
pub async fn stage_files(
    repo_id: String,
    paths: Vec<String>,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        git::stage::stage(&repo, &paths)?;
        git::status::read_status(&repo)
    })
    .await
}

#[tauri::command]
pub async fn unstage_files(
    repo_id: String,
    paths: Vec<String>,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        git::stage::unstage(&repo, &paths)?;
        git::status::read_status(&repo)
    })
    .await
}

/// Discards the changes of `paths` (the UI asks first): back to HEAD, or to
/// the Trash for files HEAD doesn't have. Returns the new status.
#[tauri::command]
pub async fn discard_changes(
    repo_id: String,
    paths: Vec<String>,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        git::discard::discard(&repo, &paths)?;
        git::status::read_status(&repo)
    })
    .await
}

/// Resolves conflicted files: take ours / theirs, or mark as resolved (fails
/// with kind "conflictMarkers" if markers are left, unless `force`).
#[tauri::command]
pub async fn resolve_conflicts(
    repo_id: String,
    paths: Vec<String>,
    choice: ConflictChoice,
    force: bool,
    store: State<'_, RepoStore>,
) -> Result<RepoStatus, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || {
        git::conflict::resolve(&repo, &paths, choice, force)?;
        git::status::read_status(&repo)
    })
    .await
}

/// The three versions of a conflicted file plus the file with markers, for
/// the conflict solver window.
#[tauri::command]
pub async fn get_conflict_versions(
    repo_id: String,
    path: String,
    store: State<'_, RepoStore>,
) -> Result<ConflictVersions, AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || git::conflict::versions(&repo, &path)).await
}

/// Saves the solver's result; with `mark_resolved`, stages it too (fails
/// with kind "conflictMarkers" if markers are left, unless `force`).
#[tauri::command]
pub async fn save_conflict_result(
    repo_id: String,
    path: String,
    content: String,
    mark_resolved: bool,
    force: bool,
    store: State<'_, RepoStore>,
) -> Result<(), AppError> {
    let repo = store.repo_path(&repo_id)?;
    run_blocking(move || git::conflict::save(&repo, &path, &content, mark_resolved, force)).await
}
