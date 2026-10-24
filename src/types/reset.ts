// Mirror ResetMode and ResetPreview in src-tauri/src/models.rs.

export type ResetMode = "soft" | "mixed" | "hard";

export interface ResetPreview {
  /** The current branch, which is moved. */
  branch: string;
  /** Short name of the target: "1234abcd", "feature", "origin/x", "v1.0". */
  target: string;
  /** The target is the current commit: only the files can change. */
  isHead: boolean;
  /** Commits the branch no longer contains afterwards. */
  leftBehind: number;
  /** How many of those are already pushed. */
  pushed: number;
  /** Files with staged or unstaged changes (Hard discards those changes). */
  changedFiles: number;
}
