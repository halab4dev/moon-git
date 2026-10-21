use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, ErrorKind};
use crate::models::{RepoEntry, RepoGroup, RepoInfo, RepoList};

/// 1: repos only. 2: adds groups and RepoEntry.group_id.
const FILE_VERSION: u32 = 2;

/// Shape of repos.json.
#[derive(Serialize, Deserialize)]
struct RepoFile {
    version: u32,
    // Version 1 files have no "groups" key: default to an empty list.
    #[serde(default)]
    groups: Vec<RepoGroup>,
    repos: Vec<RepoEntry>,
}

/// The repository list, kept in memory and written to disk on every change.
/// Tauri shares one instance between all commands, so the list sits behind a Mutex.
pub struct RepoStore {
    file: PathBuf,
    data: Mutex<RepoList>,
}

impl RepoStore {
    /// Loads the list from `file`. A missing file just means "no repos yet".
    pub fn open(file: PathBuf) -> RepoStore {
        let data = match read_file(&file) {
            Ok(data) => data,
            Err(err) => {
                // Don't refuse to start over a broken file; set it aside so nothing is lost.
                let backup = file.with_extension("json.broken");
                eprintln!("{} Moving it to {}.", err.message, backup.display());
                if let Err(rename_err) = fs::rename(&file, &backup) {
                    eprintln!("Could not move {}: {rename_err}", file.display());
                }
                RepoList::default()
            }
        };
        RepoStore {
            file,
            data: Mutex::new(data),
        }
    }

    pub fn list(&self) -> Result<RepoList, AppError> {
        let data = self.lock()?;
        Ok(data.clone())
    }

    /// Folder of a saved repository, looked up by id.
    pub fn repo_path(&self, id: &str) -> Result<PathBuf, AppError> {
        let data = self.lock()?;
        match data.repos.iter().find(|r| r.id == id) {
            Some(repo) => Ok(PathBuf::from(&repo.path)),
            None => Err(AppError::new(
                ErrorKind::NotFound,
                "Repository not found. It may have been removed from the list.".to_string(),
            )),
        }
    }

    pub fn add_repo(
        &self,
        info: RepoInfo,
        group_id: Option<String>,
    ) -> Result<RepoEntry, AppError> {
        self.update(|list| list.add_repo(info, group_id))
    }

    pub fn remove_repo(&self, id: &str) -> Result<RepoList, AppError> {
        self.update(|list| {
            list.remove_repo(id);
            Ok(list.clone())
        })
    }

    pub fn move_repo(&self, repo_id: &str, group_id: Option<String>) -> Result<RepoList, AppError> {
        self.update(|list| {
            list.move_repo(repo_id, group_id)?;
            Ok(list.clone())
        })
    }

    pub fn create_group(
        &self,
        name: &str,
        parent_id: Option<String>,
    ) -> Result<RepoList, AppError> {
        self.update(|list| {
            list.create_group(name, parent_id)?;
            Ok(list.clone())
        })
    }

    pub fn rename_group(&self, id: &str, name: &str) -> Result<RepoList, AppError> {
        self.update(|list| {
            list.rename_group(id, name)?;
            Ok(list.clone())
        })
    }

    pub fn delete_group(&self, id: &str) -> Result<RepoList, AppError> {
        self.update(|list| {
            list.delete_group(id)?;
            Ok(list.clone())
        })
    }

    /// Applies `change` to a copy of the list, saves the copy, then swaps it in.
    /// If `change` or the save fails, neither memory nor disk is modified.
    fn update<T>(
        &self,
        change: impl FnOnce(&mut RepoList) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let mut data = self.lock()?;
        let mut updated = data.clone();
        let result = change(&mut updated)?;
        write_file(&self.file, &updated)?;
        *data = updated;
        Ok(result)
    }

    // A "poisoned" lock means a command panicked mid-update. Report it instead of panicking too.
    fn lock(&self) -> Result<MutexGuard<'_, RepoList>, AppError> {
        self.data.lock().map_err(|_| {
            AppError::new(
                ErrorKind::Storage,
                "The repository list is in a bad state. Please restart the app.".to_string(),
            )
        })
    }
}

fn read_file(file: &Path) -> Result<RepoList, AppError> {
    let text = match fs::read_to_string(file) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(RepoList::default()),
        Err(err) => return Err(storage_error("read", file, err)),
    };

    let parsed: RepoFile = serde_json::from_str(&text).map_err(|err| {
        AppError::new(
            ErrorKind::Storage,
            format!("{} is not valid: {err}.", file.display()),
        )
    })?;
    Ok(RepoList {
        groups: parsed.groups,
        repos: parsed.repos,
    })
}

fn write_file(file: &Path, list: &RepoList) -> Result<(), AppError> {
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir).map_err(|err| storage_error("create the folder for", file, err))?;
    }

    let data = RepoFile {
        version: FILE_VERSION,
        groups: list.groups.clone(),
        repos: list.repos.clone(),
    };
    let text = serde_json::to_string_pretty(&data).map_err(|err| {
        AppError::new(
            ErrorKind::Storage,
            format!("Could not encode the repository list: {err}"),
        )
    })?;

    // Write a temp file, then rename over the real one: a crash mid-write
    // can never leave a half-written repos.json behind.
    let tmp = file.with_extension("json.tmp");
    fs::write(&tmp, text).map_err(|err| storage_error("write", &tmp, err))?;
    fs::rename(&tmp, file).map_err(|err| storage_error("replace", file, err))?;
    Ok(())
}

fn storage_error(action: &str, file: &Path, err: io::Error) -> AppError {
    AppError::new(
        ErrorKind::Storage,
        format!("Could not {action} {}: {err}", file.display()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn info(root: &str) -> RepoInfo {
        RepoInfo {
            root: root.to_string(),
            name: Path::new(root)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
        }
    }

    #[test]
    fn missing_file_gives_empty_list() {
        let dir = TempDir::new().unwrap();
        let store = RepoStore::open(dir.path().join("repos.json"));
        assert_eq!(store.list().unwrap(), RepoList::default());
    }

    #[test]
    fn changes_survive_reopen() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("repos.json");

        let store = RepoStore::open(file.clone());
        let added = store.add_repo(info("/work/my app"), None).unwrap();
        let saved = store.create_group("Work", None).unwrap();
        assert_eq!(added.name, "my app");

        assert_eq!(RepoStore::open(file).list().unwrap(), saved);
    }

    #[test]
    fn failed_change_leaves_list_untouched() {
        let dir = TempDir::new().unwrap();
        let store = RepoStore::open(dir.path().join("repos.json"));
        store.add_repo(info("/work/a"), None).unwrap();
        let before = store.list().unwrap();

        assert!(store.add_repo(info("/work/a"), None).is_err());
        assert_eq!(store.list().unwrap(), before);
    }

    #[test]
    fn reads_version_1_file() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("repos.json");
        fs::write(
            &file,
            r#"{ "version": 1, "repos": [ { "id": "1", "name": "a", "path": "/work/a" } ] }"#,
        )
        .unwrap();

        let list = RepoStore::open(file).list().unwrap();

        assert!(list.groups.is_empty());
        assert_eq!(list.repos.len(), 1);
        assert_eq!(list.repos[0].group_id, None);
    }

    #[test]
    fn creates_missing_data_folder() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("not").join("yet").join("repos.json");
        RepoStore::open(file.clone())
            .add_repo(info("/work/a"), None)
            .unwrap();
        assert!(file.is_file());
    }

    #[test]
    fn broken_file_is_moved_aside() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("repos.json");
        fs::write(&file, "this is not json").unwrap();

        let store = RepoStore::open(file.clone());

        assert_eq!(store.list().unwrap(), RepoList::default());
        assert!(dir.path().join("repos.json.broken").is_file());
        assert!(!file.exists());
    }
}
