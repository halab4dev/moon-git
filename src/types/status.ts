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

export interface RepoStatus {
  staged: FileChange[];
  unstaged: FileChange[];
  untracked: FileChange[];
  conflicted: FileChange[];
}
