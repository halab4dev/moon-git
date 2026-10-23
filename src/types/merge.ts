// Mirror MergeMode and MergePreview in src-tauri/src/models.rs.

/** fastForward: only move the branch; mergeCommit: --no-ff; workingTree: stop before the commit. */
export type MergeMode = "fastForward" | "mergeCommit" | "workingTree";

export interface MergePreview {
  /** The current branch, which receives the merge. */
  branch: string;
  /** Short name of what is merged: "feature", "origin/x", "v1.0", "1234abcd". */
  source: string;
  /** Everything in `source` is already in the branch. */
  upToDate: boolean;
  /** The branch has no commits of its own, so it can just move forward. */
  canFastForward: boolean;
}
