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
}
