import { invoke } from "@tauri-apps/api/core";
import type { RepoEntry, RepoList } from "../types/repo";

// Typed wrappers so components never call invoke() with raw strings.
// Argument keys must match the Rust parameter names (camelCase here,
// snake_case in Rust: groupId <-> group_id).
// On failure the promise rejects with an AppError (see api/errors.ts).

export function listRepos(): Promise<RepoList> {
  return invoke<RepoList>("list_repos");
}

export function addRepo(path: string, groupId: string | null): Promise<RepoEntry> {
  return invoke<RepoEntry>("add_repo", { path, groupId });
}

export function removeRepo(id: string): Promise<RepoList> {
  return invoke<RepoList>("remove_repo", { id });
}

export function moveRepo(repoId: string, groupId: string | null): Promise<RepoList> {
  return invoke<RepoList>("move_repo", { repoId, groupId });
}
