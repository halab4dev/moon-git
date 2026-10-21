use std::path::Path;

use crate::error::AppError;
use crate::git;
use crate::models::RepoInfo;

#[tauri::command]
pub fn check_repo(path: String) -> Result<RepoInfo, AppError> {
    git::repo::check_repo(Path::new(&path))
}
