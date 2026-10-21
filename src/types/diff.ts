// Mirror DiffMode and FileDiff in src-tauri/src/models.rs.

/** unstaged: Index -> Working Tree, staged: HEAD -> Index, conflict: HEAD -> Working Tree */
export type DiffMode = "unstaged" | "staged" | "conflict";

export interface FileDiff {
  oldLabel: string;
  newLabel: string;
  /** null = that version does not exist, or the file is binary / too large. */
  oldText: string | null;
  newText: string | null;
  binary: boolean;
  tooLarge: boolean;
}
