// Mirrors HeadCommit in src-tauri/src/models.rs.

export interface HeadCommit {
  message: string;
  /** Already on a remote-tracking branch: amending rewrites published history. */
  pushed: boolean;
}
