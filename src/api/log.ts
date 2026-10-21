import { invoke } from "@tauri-apps/api/core";
import type { LogPage } from "../types/log";

/** Newest first: `limit` (max 2000) commits after the first `skip`. */
export function getLog(repoId: string, skip: number, limit: number): Promise<LogPage> {
  return invoke<LogPage>("get_log", { repoId, skip, limit });
}
