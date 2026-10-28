// Mirrors `AppError` in src-tauri/src/error.rs.
// "unknown" exists only on the TS side, for errors that did not come from Rust.
export type ErrorKind =
  | "gitNotFound"
  | "pathNotFound"
  | "notARepository"
  | "gitFailed"
  | "io"
  | "repoAlreadyAdded"
  | "storage"
  | "invalidInput"
  | "notFound"
  | "cancelled"
  | "authRequired"
  | "unexpectedOutput"
  | "identityUnknown"
  | "conflict"
  | "pushRejected"
  | "notMerged"
  | "conflictMarkers"
  | "unknown";

export interface AppError {
  kind: ErrorKind;
  message: string;
  /** "authRequired" only: the server that wants a login, e.g. "github.com". */
  server?: string;
}
