import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { GitProgress } from "../types/progress";
import type { RepoEntry } from "../types/repo";

// Must match PROGRESS_EVENT in src-tauri/src/commands/clone.rs.
const PROGRESS_EVENT = "git-progress";

/** Resolves when the clone has finished and the repo was added to the list. */
export function cloneRepo(
  opId: string,
  url: string,
  parentDir: string,
  folderName: string,
  groupId: string | null,
): Promise<RepoEntry> {
  return invoke<RepoEntry>("clone_repo", { opId, url, parentDir, folderName, groupId });
}

export function cancelOperation(opId: string): Promise<void> {
  return invoke<void>("cancel_operation", { opId });
}

export function suggestFolderName(url: string): Promise<string> {
  return invoke<string>("suggest_folder_name", { url });
}

/**
 * Calls `handler` for every progress line of every long git command.
 * Returns a function that stops listening; always call it when done.
 */
export function onGitProgress(handler: (progress: GitProgress) => void): Promise<UnlistenFn> {
  return listen<GitProgress>(PROGRESS_EVENT, (event) => handler(event.payload));
}
