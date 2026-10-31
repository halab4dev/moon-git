// Mirrors `Identity` in src-tauri/src/models.rs (step 14).
export interface Identity {
  /** The repository's own setting; null = not set. */
  name: string | null;
  email: string | null;
  /** What applies without it (~/.gitconfig, or the machine-wide config). */
  inheritedName: string | null;
  inheritedEmail: string | null;
}

