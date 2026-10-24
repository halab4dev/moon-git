import { invoke } from "@tauri-apps/api/core";
import type { ConflictChoice, ConflictVersions, RepoStatus } from "../types/status";

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

/**
 * Conflicted files: take ours / theirs, or mark as resolved. Rejects with
 * kind "conflictMarkers" when markers are left, unless `force`.
 */
export function resolveConflicts(
  repoId: string,
  paths: string[],
  choice: ConflictChoice,
  force: boolean,
): Promise<RepoStatus> {
  return invoke<RepoStatus>("resolve_conflicts", { repoId, paths, choice, force });
}

/** The three versions of a conflicted file plus the file with markers. */
export function getConflictVersions(repoId: string, path: string): Promise<ConflictVersions> {
  return invoke<ConflictVersions>("get_conflict_versions", { repoId, path });
}

/** Writes the solver's result; with `markResolved`, stages it too (kind "conflictMarkers" if markers are left, unless `force`). */
export function saveConflictResult(
  repoId: string,
  path: string,
  content: string,
  markResolved: boolean,
  force: boolean,
): Promise<void> {
  return invoke<void>("save_conflict_result", { repoId, path, content, markResolved, force });
}
