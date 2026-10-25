// Mirrors PullMode in src-tauri/src/models.rs.
export type PullMode = "fastForward" | "merge" | "rebase";

/** Mirrors ForcePushPreview in src-tauri/src/models.rs. */
export interface ForcePushPreview {
  /** The current branch, e.g. "main". */
  branch: string;
  /** Where it goes, e.g. "origin/main". */
  upstream: string;
  /** Local commits that the server branch doesn't have. */
  ahead: number;
  /** Commits on the server branch (as of the last fetch) that the push removes. */
  lostCount: number;
  /** The newest of those, at most 10. */
  lost: { id: string; subject: string }[];
  /** Server branch at the last fetch; the push fails if the server moved since. */
  expected: string;
}
