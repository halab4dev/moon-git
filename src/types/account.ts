// Mirrors `Account` in src-tauri/src/models.rs. The token is never here:
// it only goes from the Sign in dialog into the OS keyring.
export interface Account {
  id: string;
  /** "github.com", or "host:port". */
  host: string;
  username: string;
}
