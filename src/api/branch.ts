import { invoke } from "@tauri-apps/api/core";
import type { RepoStatus } from "../types/status";

// Both resolve with the new status (the current branch changed).

/** `fullName`: "refs/heads/x", or "refs/remotes/origin/x" to get a local branch tracking it. */
export function checkoutBranch(repoId: string, fullName: string): Promise<RepoStatus> {
  return invoke<RepoStatus>("checkout_branch", { repoId, fullName });
}

/** `start`: commit id or full ref name; null = the current HEAD. */
export function createBranch(
  repoId: string,
  name: string,
  start: string | null,
  checkOut: boolean,
): Promise<RepoStatus> {
  return invoke<RepoStatus>("create_branch", { repoId, name, start, checkOut });
}
