// Mirror the history structs in src-tauri/src/models.rs.

export type RefKind = "branch" | "remote" | "tag" | "head";

export interface CommitRef {
  kind: RefKind;
  /** "main", "origin/main", "v1.0", or "HEAD" (detached). */
  name: string;
  /** The checked-out branch, or the detached HEAD. */
  current: boolean;
}

export interface CommitSummary {
  id: string;
  /** Empty for the first commit, two or more for a merge. */
  parents: string[];
  authorName: string;
  authorEmail: string;
  /** Unix seconds (UTC). */
  authorTime: number;
  subject: string;
  refs: CommitRef[];
}

export interface LogPage {
  commits: CommitSummary[];
  hasMore: boolean;
}
