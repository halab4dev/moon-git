import { invoke } from "@tauri-apps/api/core";

/**
 * Fetches from all remotes of the repository. Progress: `onGitProgress`
 * lines tagged with `opId`; stop it with `cancelOperation(opId)`.
 */
export function fetchRepo(opId: string, repoId: string): Promise<void> {
  return invoke<void>("fetch_repo", { opId, repoId });
}
