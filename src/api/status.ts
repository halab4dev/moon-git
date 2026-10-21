import { invoke } from "@tauri-apps/api/core";
import type { RepoStatus } from "../types/status";

export function getStatus(repoId: string): Promise<RepoStatus> {
  return invoke<RepoStatus>("get_status", { repoId });
}
