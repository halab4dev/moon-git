// Mirrors RebasePreview in src-tauri/src/models.rs.

export interface RebasePreview {
  /** The current branch, whose commits are replayed. */
  branch: string;
  /** Short name of the target: "feature", "origin/x", "v1.0", "1234abcd". */
  onto: string;
  /** The branch already starts from the target: nothing to do. */
  upToDate: boolean;
  /** Commits that get replayed; 0 = the branch just moves forward. */
  commits: number;
  /** How many of those are already pushed (the next push would need --force). */
  pushed: number;
}
