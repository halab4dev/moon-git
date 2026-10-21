import { invoke } from "@tauri-apps/api/core";
import type { LogPage, RefInfo } from "../types/log";

/**
 * Newest first: `limit` (max 2000) commits after the first `skip`.
 * `refs`: full ref names to show the history of; null = all branches, remotes and tags.
 * `filter`: only commits matching this text (hide mode); null = no filter.
 */
export function getLog(
  repoId: string,
  skip: number,
  limit: number,
  refs: string[] | null,
  filter: string | null,
): Promise<LogPage> {
  return invoke<LogPage>("get_log", { repoId, skip, limit, refs, filter });
}

export function listRefs(repoId: string): Promise<RefInfo[]> {
  return invoke<RefInfo[]>("list_refs", { repoId });
}

/** Ids of every commit matching `text` (id prefix or message), newest first. */
export function findCommits(repoId: string, refs: string[] | null, text: string): Promise<string[]> {
  return invoke<string[]>("find_commits", { repoId, refs, text });
}
