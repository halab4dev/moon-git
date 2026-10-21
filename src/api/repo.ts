import { invoke } from "@tauri-apps/api/core";
import type { RepoInfo } from "../types/repo";

// Typed wrappers so components never call invoke() with raw strings.
// Argument keys must match the Rust parameter names.
// On failure the promise rejects with an AppError (see api/errors.ts).
export function checkRepo(path: string): Promise<RepoInfo> {
  return invoke<RepoInfo>("check_repo", { path });
}
