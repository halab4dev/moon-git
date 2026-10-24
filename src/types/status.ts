// Mirror the status structs in src-tauri/src/models.rs.

export type ChangeKind =
  | "added"
  | "modified"
  | "deleted"
  | "renamed"
  | "copied"
  | "typeChanged"
  | "unmerged"
  | "untracked";

export interface FileChange {
  /** Relative to the repository root, "/" separators. */
  path: string;
  /** Previous path, for renamed and copied files. */
  origPath: string | null;
  kind: ChangeKind;
}

export interface BranchInfo {
  /** null when HEAD is detached. */
  name: string | null;
  /** Current commit id; null before the first commit. */
  commit: string | null;
  /** e.g. "origin/main"; null when the branch tracks nothing. */
  upstream: string | null;
  /** The upstream is configured but was deleted on the server. */
  upstreamGone: boolean;
  ahead: number;
  behind: number;
}

export interface RepoStatus {
  branch: BranchInfo;
  staged: FileChange[];
  unstaged: FileChange[];
  untracked: FileChange[];
  conflicted: FileChange[];
  /** Set while a merge waits for its commit; null otherwise. */
  merge: MergeState | null;
  /** Set while a rebase waits halfway; null otherwise. */
  rebase: RebaseState | null;
  /** Set while a cherry-pick waits after conflicts; null otherwise. */
  cherryPick: CherryPickState | null;
}

export interface CherryPickState {
  /** Full id of the commit being picked. */
  commit: string;
  subject: string;
}

export interface RebaseState {
  /** The branch being rebased; "HEAD" if it started detached. */
  branch: string;
  /** Full id of the commit it is rebased onto. */
  onto: string;
  /** Which commit is being replayed (1-based), of how many. */
  step: number;
  total: number;
  /** "1234abcd subject" of the commit where it stopped. */
  stoppedAt: string | null;
}

export interface MergeState {
  /** The message git prepared for the merge commit; may be empty. */
  message: string;
}

/** Mirrors ConflictChoice in models.rs. */
export type ConflictChoice = "ours" | "theirs" | "resolved";

/** Mirrors ConflictVersions in models.rs. Texts are null when missing or not editable as text. */
export interface ConflictVersions {
  base: string | null;
  ours: string | null;
  theirs: string | null;
  /** The file with git's conflict markers. */
  merged: string | null;
  binary: boolean;
  tooLarge: boolean;
  /** One side deleted the file. */
  missingSide: boolean;
}
