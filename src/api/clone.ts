import { invoke } from "@tauri-apps/api/core";
import type { RepoEntry } from "../types/repo";

// Progress and cancelling: see operations.ts.

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

export function suggestFolderName(url: string): Promise<string> {
  return invoke<string>("suggest_folder_name", { url });
}
