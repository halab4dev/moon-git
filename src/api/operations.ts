import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { GitProgress } from "../types/progress";

// Shared by every long-running git command (clone, fetch, later pull/push).

// Must match PROGRESS_EVENT in src-tauri/src/commands/mod.rs.
const PROGRESS_EVENT = "git-progress";

export function cancelOperation(opId: string): Promise<void> {
  return invoke<void>("cancel_operation", { opId });
}

/**
 * Calls `handler` for every progress line of every long git command.
 * Returns a function that stops listening; always call it when done.
 */
export function onGitProgress(handler: (progress: GitProgress) => void): Promise<UnlistenFn> {
  return listen<GitProgress>(PROGRESS_EVENT, (event) => handler(event.payload));
}
