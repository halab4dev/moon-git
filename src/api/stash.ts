import { invoke } from "@tauri-apps/api/core";
import type { StashEntry } from "../types/stash";
import type { RepoStatus } from "../types/status";

/** Resolves with the new status. An empty `message` lets git write "WIP on …". */
export function createStash(repoId: string, message: string, includeUntracked: boolean): Promise<RepoStatus> {
  return invoke<RepoStatus>("create_stash", { repoId, message, includeUntracked });
}

/** Newest first. */
export function listStashes(repoId: string): Promise<StashEntry[]> {
  return invoke<StashEntry[]>("list_stashes", { repoId });
}
