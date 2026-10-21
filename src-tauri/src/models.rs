// Data structures returned to the frontend.
// Each struct here has a matching TypeScript interface in src/types/.

use serde::{Deserialize, Serialize};

/// Result of checking a folder with git (see git/repo.rs).
#[derive(Debug, Serialize)]
// Rust uses snake_case, TS uses camelCase: let serde rename fields on output.
#[serde(rename_all = "camelCase")]
pub struct RepoInfo {
    /// Absolute path of the working tree root, as reported by git.
    pub root: String,
    /// Folder name of the root, used as the default display name.
    pub name: String,
}

/// A repository saved in the app's list. Also the on-disk format in repos.json.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoEntry {
    pub id: String,
    pub name: String,
    pub path: String,
}
