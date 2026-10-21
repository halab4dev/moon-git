import { invoke } from "@tauri-apps/api/core";
import type { LogPage, RefInfo } from "../types/log";

/**
 * Newest first: `limit` (max 2000) commits after the first `skip`.
 * `refs`: full ref names to show the history of; null = all branches, remotes and tags.
 */
export function getLog(repoId: string, skip: number, limit: number, refs: string[] | null): Promise<LogPage> {
  return invoke<LogPage>("get_log", { repoId, skip, limit, refs });
}

export function listRefs(repoId: string): Promise<RefInfo[]> {
  return invoke<RefInfo[]>("list_refs", { repoId });
}
