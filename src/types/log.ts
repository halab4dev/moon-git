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

/** A branch, remote branch or tag (or a detached HEAD) the history can show. */
export interface RefInfo {
  /** "refs/heads/main", or "HEAD" when detached. Sent back to choose the history. */
  fullName: string;
  /** "main", "origin/main", "v1.0", "HEAD". */
  name: string;
  kind: RefKind;
  current: boolean;
  /** Local branches only: full name of the tracked branch, e.g. "refs/remotes/origin/main". */
  upstream: string | null;
}
