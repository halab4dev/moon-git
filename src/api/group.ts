import { invoke } from "@tauri-apps/api/core";
import type { RepoList } from "../types/repo";

export function createGroup(name: string, parentId: string | null): Promise<RepoList> {
  return invoke<RepoList>("create_group", { name, parentId });
}

export function renameGroup(id: string, name: string): Promise<RepoList> {
  return invoke<RepoList>("rename_group", { id, name });
}

export function deleteGroup(id: string): Promise<RepoList> {
  return invoke<RepoList>("delete_group", { id });
}

/** Moves a group with everything inside under `parentId` (null = top level). */
export function moveGroup(id: string, parentId: string | null): Promise<RepoList> {
  return invoke<RepoList>("move_group", { id, parentId });
}
