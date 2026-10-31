// Mirrors `RemoteAccess` in src-tauri/src/models.rs (step 15a).
export interface RemoteAccess {
  /** Private key used for SSH; null = this computer's SSH settings. */
  sshKey: string | null;
  /** A core.sshCommand not written by MoonGit (set by hand), if any. */
  customSshCommand: string | null;
  /** HTTPS: this computer's credential helpers instead of MoonGit's accounts. */
  systemCredentials: boolean;
  /** Remotes using SSH / HTTPS ("origin", or "origin (push)" for a push URL only). */
  sshRemotes: string[];
  httpsRemotes: string[];
}
