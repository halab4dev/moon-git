import { invoke } from "@tauri-apps/api/core";
import type { MergeMode, MergePreview } from "../types/merge";
import type { RepoStatus } from "../types/status";

// `source`: full ref name ("refs/heads/x", "refs/remotes/origin/x", "refs/tags/v1") or a commit id.

export function getMergePreview(repoId: string, source: string): Promise<MergePreview> {
  return invoke<MergePreview>("get_merge_preview", { repoId, source });
}

/** Resolves with the new status; rejects with kind "conflict" when files conflict. */
export function mergeIntoCurrent(repoId: string, source: string, mode: MergeMode): Promise<RepoStatus> {
  return invoke<RepoStatus>("merge_into_current", { repoId, source, mode });
}

/** `git merge --abort`; resolves with the new status. */
export function abortMerge(repoId: string): Promise<RepoStatus> {
  return invoke<RepoStatus>("abort_merge", { repoId });
}
