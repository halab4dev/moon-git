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
    /// Set while a merge waits for its commit (step 21b): after "Merge to
    /// Working Tree", or after a merge stopped on conflicts.
    pub merge: Option<MergeState>,
    /// Set while a rebase waits halfway, e.g. on conflicts (step 22b).
    pub rebase: Option<RebaseState>,
    /// Set while a cherry-pick waits after conflicts (step 23).
    pub cherry_pick: Option<CherryPickState>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CherryPickState {
    /// Full id of the commit being picked.
    pub commit: String,
    /// Its subject (first line of the prepared message).
    pub subject: String,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RebaseState {
    /// The branch being rebased ("main"); "HEAD" if it started detached.
    pub branch: String,
    /// The commit the branch is rebased onto (full id).
    pub onto: String,
    /// Which commit is being replayed (1-based), of how many.
    pub step: u32,
    pub total: u32,
    /// The commit where it stopped: "1234abcd subject". None if unknown.
    pub stopped_at: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeState {
    /// The message git prepared for the merge commit (.git/MERGE_MSG),
    /// without git's "#" comment lines. Empty if there is none.
    pub message: String,
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

/// How to merge (step 21). Sent by the frontend.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MergeMode {
    /// Only move the branch forward; fails if it has commits of its own.
    FastForward,
    /// Always create a merge commit (--no-ff).
    MergeCommit,
    /// Merge, but stop before the commit (--no-ff --no-commit).
    WorkingTree,
}

/// What a merge would do, shown in the merge dialog before choosing.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MergePreview {
    /// The current branch, which receives the merge.
    pub branch: String,
    /// Short name of what is merged: "feature", "origin/x", "v1.0", "1234abcd".
    pub source: String,
    /// Everything in `source` is already in the branch.
    pub up_to_date: bool,
    /// The branch has no commits of its own, so it can just move forward.
    pub can_fast_forward: bool,
}

/// What rebasing the current branch would do (step 22), shown before starting.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RebasePreview {
    /// The current branch, whose commits are replayed.
    pub branch: String,
    /// Short name of the target: "feature", "origin/x", "v1.0", "1234abcd".
    pub onto: String,
    /// The branch already starts from the target: nothing to do.
    pub up_to_date: bool,
    /// Commits of the branch that are not in the target (they get replayed).
    /// 0 = the branch just moves forward to the target.
    pub commits: u32,
    /// How many of those are already on a remote branch. Rewriting them
    /// means the next push needs --force.
    pub pushed: u32,
}

/// What to do with a rebase that waits halfway (step 22b). Sent by the frontend.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RebaseAction {
    /// Commit the resolved commit and replay the rest.
    Continue,
    /// Leave the stopped commit out, and replay the rest.
    Skip,
    /// Undo the whole rebase: the branch goes back to where it was.
    Abort,
}

/// How far a reset goes (step 22c). Sent by the frontend.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResetMode {
    /// Move the branch; keep index and working tree.
    Soft,
    /// Move the branch and reset the index; keep the working tree.
    Mixed,
    /// Move the branch and reset index and working tree.
    Hard,
}

/// What a reset would do, shown in the reset dialog.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetPreview {
    /// The current branch, which is moved.
    pub branch: String,
    /// Short name of the target: "1234abcd", "feature", "origin/x", "v1.0".
    pub target: String,
    /// The target is the current commit (only the files can change).
    pub is_head: bool,
    /// Commits on the branch now that it no longer contains afterwards.
    pub left_behind: u32,
    /// How many of those are already pushed (the next push would need --force).
    pub pushed: u32,
    /// Files with staged or unstaged changes: Hard throws those changes away.
    pub changed_files: u32,
}

/// How to cherry-pick (step 23). Sent by the frontend.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CherryPickMode {
    /// A new commit with the same changes, message and author.
    Commit,
    /// Only the changes, staged, without committing (--no-commit).
    WorkingTree,
}

/// What to do with a cherry-pick that waits after conflicts. Sent by the frontend.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CherryPickAction {
    /// Commit the resolved result.
    Continue,
    /// Undo the cherry-pick.
    Abort,
}

/// How to resolve a conflicted file (step 24a). Sent by the frontend.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConflictChoice {
    /// The whole file as on our side (the current branch; during a rebase:
    /// the branch rebased onto).
    Ours,
    /// The whole file as on their side (the merged branch, the picked
    /// commit; during a rebase: your commit being replayed).
    Theirs,
    /// Fixed by hand: stage the file as it is now.
    Resolved,
}

/// What the conflict solver window shows for one file (step 24b). The texts
/// are None when that version does not exist, or the file can't be edited
/// as text (binary, not UTF-8, too large).
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictVersions {
    /// The common ancestor (stage 1); None if both sides added the file.
    pub base: Option<String>,
    /// Our version (stage 2).
    pub ours: Option<String>,
    /// Their version (stage 3).
    pub theirs: Option<String>,
    /// The file in the working tree, with git's conflict markers.
    pub merged: Option<String>,
    pub binary: bool,
    pub too_large: bool,
    /// One side deleted the file: nothing to merge line by line.
    pub missing_side: bool,
}

/// The sidebar's branch list (step 25).
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchList {
    pub local: Vec<LocalBranch>,
    /// One group per remote, in the order of `git remote` (by name).
    pub remotes: Vec<RemoteGroup>,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalBranch {
    /// "main", "feature/x".
    pub name: String,
    /// "refs/heads/main".
    pub full_name: String,
    pub current: bool,
    /// The branch it tracks, e.g. "origin/main"; None when it tracks nothing.
    pub upstream: Option<String>,
    /// Commits here and not on the upstream, and the other way round.
    pub ahead: u32,
    pub behind: u32,
    /// The upstream is configured but was deleted on the server.
    pub upstream_gone: bool,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteGroup {
    /// "origin", "upstream".
    pub name: String,
    /// As written in the config; None for refs left over from a removed remote.
    pub url: Option<String>,
    pub branches: Vec<RemoteBranch>,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteBranch {
    /// Without the remote: "main", "feature/x".
    pub name: String,
    /// "refs/remotes/origin/main".
    pub full_name: String,
}

/// What a force push of the current branch would do (step 27), shown in the
/// confirmation before anything is sent.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForcePushPreview {
    /// The current branch, e.g. "main".
    pub branch: String,
    /// Where it goes, e.g. "origin/main".
    pub upstream: String,
    /// Local commits that the server branch doesn't have.
    pub ahead: u32,
    /// Commits on the server branch (as of the last fetch) that the push
    /// removes from it.
    pub lost_count: u32,
    /// The newest of those, at most 10.
    pub lost: Vec<LostCommit>,
    /// Full id of the server branch as of the last fetch. The push only
    /// succeeds if the server still has exactly this (--force-with-lease).
    pub expected: String,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LostCommit {
    pub id: String,
    pub subject: String,
}

/// A saved login for an HTTPS server (step 13a). The token is not here: it
/// lives in the OS keyring under `id`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    /// "github.com", or "host:port".
    pub host: String,
    pub username: String,
}
