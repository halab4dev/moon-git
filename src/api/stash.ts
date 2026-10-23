import { invoke } from "@tauri-apps/api/core";
import type { StashAction, StashEntry } from "../types/stash";
import type { RepoStatus } from "../types/status";

/** Resolves with the new status. An empty `message` lets git write "WIP on …". */
export function createStash(repoId: string, message: string, includeUntracked: boolean): Promise<RepoStatus> {
  return invoke<RepoStatus>("create_stash", { repoId, message, includeUntracked });
}

/** Newest first. */
export function listStashes(repoId: string): Promise<StashEntry[]> {
  return invoke<StashEntry[]>("list_stashes", { repoId });
}

/**
 * Apply, pop or drop the stash with commit `id` (not its index, which shifts).
 * Resolves with the new status; rejects with kind "conflict" when applying
 * left conflicts (the stash is then kept, even for pop).
 */
export function runStashAction(repoId: string, id: string, action: StashAction): Promise<RepoStatus> {
  return invoke<RepoStatus>("run_stash_action", { repoId, id, action });
}
