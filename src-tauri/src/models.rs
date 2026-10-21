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
    /// None = top level. Missing in version 1 files, which serde reads as None.
    #[serde(default)]
    pub group_id: Option<String>,
}

/// A folder in the sidebar tree. Groups nest through `parent_id`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoGroup {
    pub id: String,
    pub name: String,
    /// None = top level.
    pub parent_id: Option<String>,
}

/// Everything shown in the sidebar. Stored flat; the frontend builds the tree.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct RepoList {
    pub groups: Vec<RepoGroup>,
    pub repos: Vec<RepoEntry>,
}

/// Payload of the "git-progress" event, sent for every line a long git command prints.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitProgress {
    pub op_id: String,
    pub line: String,
    /// e.g. "Receiving objects"; None when the line has no percentage.
    pub phase: Option<String>,
    pub percent: Option<u8>,
}
