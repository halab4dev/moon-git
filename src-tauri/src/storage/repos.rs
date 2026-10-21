use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, ErrorKind};
use crate::models::{RepoEntry, RepoInfo};

const FILE_VERSION: u32 = 1;

/// Shape of repos.json. `version` lets later steps change the format safely.
#[derive(Serialize, Deserialize)]
struct RepoFile {
    version: u32,
    repos: Vec<RepoEntry>,
}

/// The repository list, kept in memory and written to disk on every change.
/// Tauri shares one instance between all commands, so the list sits behind a Mutex.
pub struct RepoStore {
    file: PathBuf,
    repos: Mutex<Vec<RepoEntry>>,
}

impl RepoStore {
    /// Loads the list from `file`. A missing file just means "no repos yet".
    pub fn open(file: PathBuf) -> RepoStore {
        let repos = match read_file(&file) {
            Ok(repos) => repos,
            Err(err) => {
                // Don't refuse to start over a broken file; set it aside so nothing is lost.
                let backup = file.with_extension("json.broken");
                eprintln!("{} Moving it to {}.", err.message, backup.display());
                if let Err(rename_err) = fs::rename(&file, &backup) {
                    eprintln!("Could not move {}: {rename_err}", file.display());
                }
                Vec::new()
            }
        };
        RepoStore {
            file,
            repos: Mutex::new(repos),
        }
    }

    pub fn list(&self) -> Result<Vec<RepoEntry>, AppError> {
        let repos = self.lock()?;
        Ok(repos.clone())
    }

    pub fn add(&self, info: RepoInfo) -> Result<RepoEntry, AppError> {
        let mut repos = self.lock()?;

        if let Some(existing) = repos.iter().find(|r| r.path == info.root) {
            return Err(AppError::new(
                ErrorKind::RepoAlreadyAdded,
                format!("{} is already in the list.", existing.name),
            ));
        }

        let entry = RepoEntry {
            id: new_id(),
            name: info.name,
            path: info.root,
        };
        let mut updated = repos.clone();
        updated.push(entry.clone());

        // Save before touching memory, so a failed write leaves both unchanged.
        write_file(&self.file, &updated)?;
        *repos = updated;
        Ok(entry)
    }

    /// Removes the entry from the list only; the folder on disk is not touched.
    pub fn remove(&self, id: &str) -> Result<(), AppError> {
        let mut repos = self.lock()?;
        let updated: Vec<RepoEntry> = repos.iter().filter(|r| r.id != id).cloned().collect();

        write_file(&self.file, &updated)?;
        *repos = updated;
        Ok(())
    }

    // A "poisoned" lock means a command panicked mid-update. Report it instead of panicking too.
    fn lock(&self) -> Result<MutexGuard<'_, Vec<RepoEntry>>, AppError> {
        self.repos.lock().map_err(|_| {
            AppError::new(
                ErrorKind::Storage,
                "The repository list is in a bad state. Please restart the app.".to_string(),
            )
        })
    }
}

fn read_file(file: &Path) -> Result<Vec<RepoEntry>, AppError> {
    let text = match fs::read_to_string(file) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(storage_error("read", file, err)),
    };

    let parsed: RepoFile = serde_json::from_str(&text).map_err(|err| {
        AppError::new(
            ErrorKind::Storage,
            format!("{} is not valid: {err}.", file.display()),
        )
    })?;
    Ok(parsed.repos)
}

fn write_file(file: &Path, repos: &[RepoEntry]) -> Result<(), AppError> {
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir).map_err(|err| storage_error("create the folder for", file, err))?;
    }

    let data = RepoFile {
        version: FILE_VERSION,
        repos: repos.to_vec(),
    };
    let text = serde_json::to_string_pretty(&data).map_err(|err| {
        AppError::new(ErrorKind::Storage, format!("Could not encode the repository list: {err}"))
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

// Nanoseconds since 1970, in hex. Ids are created one at a time under the lock
// by user clicks, so this is unique enough without pulling in a UUID crate.
fn new_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn info(root: &str) -> RepoInfo {
        RepoInfo {
            root: root.to_string(),
            name: Path::new(root).file_name().unwrap().to_string_lossy().to_string(),
        }
    }

    #[test]
    fn missing_file_gives_empty_list() {
        let dir = TempDir::new().unwrap();
        let store = RepoStore::open(dir.path().join("repos.json"));
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn added_repo_survives_reopen() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("repos.json");

        let added = RepoStore::open(file.clone()).add(info("/work/my app")).unwrap();
        assert_eq!(added.name, "my app");

        let reopened = RepoStore::open(file).list().unwrap();
        assert_eq!(reopened, vec![added]);
    }

    #[test]
    fn rejects_duplicate_path() {
        let dir = TempDir::new().unwrap();
        let store = RepoStore::open(dir.path().join("repos.json"));
        store.add(info("/work/a")).unwrap();

        let err = store.add(info("/work/a")).unwrap_err();
        assert_eq!(err.kind, ErrorKind::RepoAlreadyAdded);
        assert_eq!(store.list().unwrap().len(), 1);
    }

    #[test]
    fn remove_deletes_only_that_entry() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("repos.json");
        let store = RepoStore::open(file.clone());
        let a = store.add(info("/work/a")).unwrap();
        let b = store.add(info("/work/b")).unwrap();

        store.remove(&a.id).unwrap();

        assert_eq!(RepoStore::open(file).list().unwrap(), vec![b]);
    }

    #[test]
    fn creates_missing_data_folder() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("not").join("yet").join("repos.json");
        RepoStore::open(file.clone()).add(info("/work/a")).unwrap();
        assert!(file.is_file());
    }

    #[test]
    fn broken_file_is_moved_aside() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("repos.json");
        fs::write(&file, "this is not json").unwrap();

        let store = RepoStore::open(file.clone());

        assert!(store.list().unwrap().is_empty());
        assert!(dir.path().join("repos.json.broken").is_file());
        assert!(!file.exists());
    }
}
