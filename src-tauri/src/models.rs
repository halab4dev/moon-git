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

/// What happened to a file, from one side of `git status` (index or working tree).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    TypeChanged,
    Unmerged,
    Untracked,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileChange {
    /// Path relative to the repository root, always with "/" separators.
    pub path: String,
    /// Previous path, for renamed and copied files.
    pub orig_path: Option<String>,
    pub kind: ChangeKind,
}

/// Result of `git status`. A file changed both in the index and in the
/// working tree appears in both `staged` and `unstaged`.
#[derive(Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoStatus {
    pub branch: BranchInfo,
    pub staged: Vec<FileChange>,
    pub unstaged: Vec<FileChange>,
    pub untracked: Vec<FileChange>,
    pub conflicted: Vec<FileChange>,
}

/// The checked-out branch and how it compares with its upstream
/// (from the "# branch.*" header lines of `git status --branch`).
#[derive(Debug, Default, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchInfo {
    /// None when HEAD is detached (a commit is checked out, not a branch).
    pub name: Option<String>,
    /// Full id of the current commit; None before the first commit.
    pub commit: Option<String>,
    /// e.g. "origin/main"; None when the branch tracks nothing.
    pub upstream: Option<String>,
    /// The upstream is configured but no longer exists (deleted on the server).
    pub upstream_gone: bool,
    /// Commits here and not on the upstream, and the other way round.
    /// Both 0 without a (living) upstream.
    pub ahead: u32,
    pub behind: u32,
}

/// Which two versions of a file the diff view compares. Sent by the frontend,
/// so it derives Deserialize (JSON "unstaged" -> DiffMode::Unstaged).
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DiffMode {
    /// Changes not staged yet: index -> working tree.
    Unstaged,
    /// Changes staged for the next commit: HEAD -> index.
    Staged,
    /// A file with merge conflicts: HEAD -> working tree (with conflict markers).
    Conflict,
}

/// The two versions of a file, ready to show side by side.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDiff {
    /// e.g. "HEAD", "Index", "Working Tree".
    pub old_label: String,
    pub new_label: String,
    /// None when that version does not exist (new or deleted file), or when
    /// the file is binary or too large to show.
    pub old_text: Option<String>,
    pub new_text: Option<String>,
    pub binary: bool,
    pub too_large: bool,
}

/// The commit HEAD points to; used to prefill the message when amending.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeadCommit {
    pub message: String,
    /// Already on a remote-tracking branch: amending it rewrites published
    /// history, and the next push would need --force.
    pub pushed: bool,
}
