import { invoke } from "@tauri-apps/api/core";
import type { ResetMode, ResetPreview } from "../types/reset";
import type { RepoStatus } from "../types/status";

// `target`: full ref name or a commit id.

export function getResetPreview(repoId: string, target: string): Promise<ResetPreview> {
  return invoke<ResetPreview>("get_reset_preview", { repoId, target });
}

/** Moves the current branch to `target`; resolves with the new status. */
export function resetCurrent(repoId: string, target: string, mode: ResetMode): Promise<RepoStatus> {
  return invoke<RepoStatus>("reset_current", { repoId, target, mode });
}
