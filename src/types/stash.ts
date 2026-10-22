// Mirrors StashEntry in src-tauri/src/models.rs.

export interface StashEntry {
  /** 0 = stash@{0}, the newest. Shifts when stashes come and go; `id` does not. */
  index: number;
  /** The stash commit: compared with `base`, the stashed changes of tracked files. */
  id: string;
  /** The commit that was checked out when stashing. */
  base: string;
  /** Commit holding the untracked files, if they were included. */
  untracked: string | null;
  /** Unix seconds. */
  time: number;
  /** "On main: <message>", or "WIP on main: <commit>" without a message. */
  message: string;
}
