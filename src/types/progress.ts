// Mirrors `GitProgress` in src-tauri/src/models.rs.
export interface GitProgress {
  opId: string;
  line: string;
  /** e.g. "Receiving objects"; null when the line has no percentage. */
  phase: string | null;
  percent: number | null;
}
