import { invoke } from "@tauri-apps/api/core";
import type { RepoEntry } from "../types/repo";

// Typed wrappers so components never call invoke() with raw strings.
// Argument keys must match the Rust parameter names.
// On failure the promise rejects with an AppError (see api/errors.ts).

export function listRepos(): Promise<RepoEntry[]> {
  return invoke<RepoEntry[]>("list_repos");
}

export function addRepo(path: string): Promise<RepoEntry> {
  return invoke<RepoEntry>("add_repo", { path });
}

export function removeRepo(id: string): Promise<void> {
  return invoke<void>("remove_repo", { id });
}
