import { invoke } from "@tauri-apps/api/core";
import type { FileDiff } from "../types/diff";
import type { CommitDetails, LogPage, RefInfo } from "../types/log";

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

export function getCommitDetails(repoId: string, commitId: string): Promise<CommitDetails> {
  return invoke<CommitDetails>("get_commit_details", { repoId, commitId });
}

/** One file of a commit against the first parent (`parentId` null for the first commit). */
export function getCommitFileDiff(
  repoId: string,
  commitId: string,
  parentId: string | null,
  path: string,
  origPath: string | null,
): Promise<FileDiff> {
  return invoke<FileDiff>("get_commit_file_diff", { repoId, commitId, parentId, path, origPath });
}
