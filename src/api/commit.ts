import { invoke } from "@tauri-apps/api/core";
import type { HeadCommit } from "../types/commit";
import type { RepoStatus } from "../types/status";

/**
 * Commits the staged files (or amends the last commit); resolves with the new status.
 * `simpleCommit`: during a merge, an ordinary commit (one parent) instead of the merge commit.
 */
export function commitChanges(
  repoId: string,
  message: string,
  amend: boolean,
  simpleCommit: boolean,
): Promise<RepoStatus> {
  return invoke<RepoStatus>("commit_changes", { repoId, message, amend, simpleCommit });
}

/** null before the first commit. */
export function getHeadCommit(repoId: string): Promise<HeadCommit | null> {
  return invoke<HeadCommit | null>("get_head_commit", { repoId });
}
