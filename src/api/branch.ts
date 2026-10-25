import { invoke } from "@tauri-apps/api/core";
import type { BranchList } from "../types/branches";
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

/** Local branch `fullName` ("refs/heads/x") gets `newName`. */
export function renameBranch(repoId: string, fullName: string, newName: string): Promise<RepoStatus> {
  return invoke<RepoStatus>("rename_branch", { repoId, fullName, newName });
}

/** Local branch. Rejects with kind "notMerged" unless `force`. */
export function deleteBranch(repoId: string, fullName: string, force: boolean): Promise<RepoStatus> {
  return invoke<RepoStatus>("delete_branch", { repoId, fullName, force });
}

/** Branch on the server, given as "refs/remotes/<remote>/<branch>". Progress tagged `opId`. */
export function deleteRemoteBranch(opId: string, repoId: string, fullName: string): Promise<RepoStatus> {
  return invoke<RepoStatus>("delete_remote_branch", { opId, repoId, fullName });
}

/** `upstream`: "refs/remotes/origin/x", or null to stop tracking. */
export function setBranchUpstream(repoId: string, fullName: string, upstream: string | null): Promise<RepoStatus> {
  return invoke<RepoStatus>("set_branch_upstream", { repoId, fullName, upstream });
}

/** Local branches, and remote branches grouped by remote (for the sidebar). */
export function listBranches(repoId: string): Promise<BranchList> {
  return invoke<BranchList>("list_branches", { repoId });
}
