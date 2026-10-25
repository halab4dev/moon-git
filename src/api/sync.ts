import { invoke } from "@tauri-apps/api/core";
import type { RepoStatus } from "../types/status";
import type { ForcePushPreview, PullMode } from "../types/sync";

// Pull and push. Fetch is in fetch.ts; progress/cancel in operations.ts.

/**
 * Second half of a pull: merges or rebases the already fetched upstream
 * into the current branch. Call `fetchRepo` first.
 */
export function pullRepo(repoId: string, mode: PullMode): Promise<RepoStatus> {
  return invoke<RepoStatus>("pull_repo", { repoId, mode });
}

/** "merge" or "rebase", from the user's git config (pull.rebase). */
export function getPullDefault(repoId: string): Promise<PullMode> {
  return invoke<PullMode>("get_pull_default", { repoId });
}

/** `newUpstream`: remote to create and track the branch on (first push only), else null. */
export function pushRepo(opId: string, repoId: string, newUpstream: string | null): Promise<void> {
  return invoke<void>("push_repo", { opId, repoId, newUpstream });
}

/** What force pushing the current branch would remove from the server. */
export function forcePushPreview(repoId: string): Promise<ForcePushPreview> {
  return invoke<ForcePushPreview>("force_push_preview", { repoId });
}

/** Force push, only if the server branch is still at `expected` (from the preview). */
export function forcePushRepo(opId: string, repoId: string, expected: string): Promise<void> {
  return invoke<void>("force_push_repo", { opId, repoId, expected });
}

export function listRemotes(repoId: string): Promise<string[]> {
  return invoke<string[]>("list_remotes", { repoId });
}
