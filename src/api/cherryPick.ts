import { invoke } from "@tauri-apps/api/core";
import type { CherryPickAction, CherryPickMode } from "../types/cherryPick";
import type { RepoStatus } from "../types/status";

/** Applies the changes of `commit` (an id) to the current branch; rejects with kind "conflict" on conflicts. */
export function cherryPickCommit(repoId: string, commit: string, mode: CherryPickMode): Promise<RepoStatus> {
  return invoke<RepoStatus>("cherry_pick_commit", { repoId, commit, mode });
}

/** Continue or abort a cherry-pick that waits after conflicts. */
export function runCherryPickAction(repoId: string, action: CherryPickAction): Promise<RepoStatus> {
  return invoke<RepoStatus>("run_cherry_pick_action", { repoId, action });
}
