import { invoke } from "@tauri-apps/api/core";
import type { DiffMode, FileDiff } from "../types/diff";

/** `origPath`: old name of a renamed file, used with mode "staged". */
export function getFileDiff(
  repoId: string,
  path: string,
  origPath: string | null,
  mode: DiffMode,
): Promise<FileDiff> {
  return invoke<FileDiff>("get_file_diff", { repoId, path, origPath, mode });
}
