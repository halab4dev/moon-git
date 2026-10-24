import { invoke } from "@tauri-apps/api/core";
import type { RebaseAction, RebasePreview } from "../types/rebase";
import type { RepoStatus } from "../types/status";

// `onto`: full ref name ("refs/heads/x", "refs/remotes/origin/x", "refs/tags/v1") or a commit id.

export function getRebasePreview(repoId: string, onto: string): Promise<RebasePreview> {
  return invoke<RebasePreview>("get_rebase_preview", { repoId, onto });
}

/** Resolves with the new status; rejects with kind "conflict" when the rebase stops on conflicts. */
export function rebaseCurrent(repoId: string, onto: string): Promise<RepoStatus> {
  return invoke<RepoStatus>("rebase_current", { repoId, onto });
}

/** Continue / skip / abort a rebase that waits halfway; resolves with the new status. */
export function runRebaseAction(repoId: string, action: RebaseAction): Promise<RepoStatus> {
  return invoke<RepoStatus>("run_rebase_action", { repoId, action });
}
