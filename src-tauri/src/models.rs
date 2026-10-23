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

/// How pulled commits are combined with local ones. Sent by the frontend.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PullMode {
    /// Only move the branch forward; fails if there are local commits.
    FastForward,
    /// Create a merge commit joining both lines of history.
    Merge,
    /// Replay the local commits on top of the pulled ones.
    Rebase,
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

/// One commit in the history list (step 16a). Details such as the full
/// message and the changed files are loaded separately when a commit is clicked.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitSummary {
    pub id: String,
    /// Parent ids: none for the first commit, two (or more) for a merge.
    pub parents: Vec<String>,
    pub author_name: String,
    pub author_email: String,
    /// Seconds since 1970-01-01 UTC (the frontend formats it in local time).
    pub author_time: i64,
    pub subject: String,
    /// Branches and tags pointing at this commit.
    pub refs: Vec<CommitRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitRef {
    pub kind: RefKind,
    /// Short name: "main", "origin/main", "v1.0", or "HEAD" (detached).
    pub name: String,
    /// The checked-out branch (or the detached HEAD).
    pub current: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RefKind {
    Branch,
    Remote,
    Tag,
    /// HEAD pointing at a commit directly, not at a branch.
    Head,
}

/// A branch, remote branch or tag (or a detached HEAD) that the history can show.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefInfo {
    /// "refs/heads/main", or "HEAD" for a detached HEAD. This is what the
    /// frontend sends back to choose the history.
    pub full_name: String,
    /// "main", "origin/main", "v1.0", "HEAD".
    pub name: String,
    pub kind: RefKind,
    /// The checked-out branch (or the detached HEAD).
    pub current: bool,
    /// Local branches only: full name of the branch it tracks,
    /// e.g. "refs/remotes/origin/main".
    pub upstream: Option<String>,
}

/// A slice of the history, newest first.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogPage {
    pub commits: Vec<CommitSummary>,
    /// More commits exist after this page.
    pub has_more: bool,
}

/// Everything shown when a commit is clicked in the History tab.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitDetails {
    pub id: String,
    pub parents: Vec<String>,
    pub author_name: String,
    pub author_email: String,
    pub author_time: i64,
    /// Who created this commit object; differs from the author after a
    /// rebase, cherry-pick or when applying someone else's patch.
    pub committer_name: String,
    pub committer_email: String,
    pub committer_time: i64,
    /// Full message: subject, blank line, body.
    pub message: String,
    /// Changed files compared with the first parent.
    pub files: Vec<FileChange>,
}

/// One stash, as listed by `git stash list` (step 20).
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StashEntry {
    /// Position: 0 = `stash@{0}`, the newest. Changes when stashes are added
    /// or dropped, so `id` is what identifies a stash.
    pub index: usize,
    /// The stash commit. Compared with `base`, it holds the stashed changes
    /// of tracked files.
    pub id: String,
    /// The commit that was checked out when stashing.
    pub base: String,
    /// Commit holding the untracked files, when they were included.
    pub untracked: Option<String>,
    /// Unix seconds.
    pub time: i64,
    /// "On main: <message>", or "WIP on main: <commit>" without a message.
    pub message: String,
}

/// What to do with a stash (step 20b). Sent by the frontend.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StashAction {
    /// Put the changes back into the working tree; keep the stash.
    Apply,
    /// Apply, then drop the stash, but only if applying went without conflicts.
    Pop,
    /// Delete the stash. Its changes are lost (the UI asks first).
    Drop,
}
