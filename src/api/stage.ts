import { invoke } from "@tauri-apps/api/core";
import type { RepoStatus } from "../types/status";

// All resolve with the new status after the change.

export function stageFiles(repoId: string, paths: string[]): Promise<RepoStatus> {
  return invoke<RepoStatus>("stage_files", { repoId, paths });
}

/** For a renamed file, pass both the new and the old path. */
export function unstageFiles(repoId: string, paths: string[]): Promise<RepoStatus> {
  return invoke<RepoStatus>("unstage_files", { repoId, paths });
}

/**
 * Back to HEAD (staged and unstaged changes alike); files HEAD doesn't have
 * go to the Trash. A renamed file: pass its new path only.
 */
export function discardChanges(repoId: string, paths: string[]): Promise<RepoStatus> {
  return invoke<RepoStatus>("discard_changes", { repoId, paths });
}
