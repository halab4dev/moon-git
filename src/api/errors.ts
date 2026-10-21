import type { AppError } from "../types/error";

// invoke() rejects with whatever Rust returned as the error, typed as unknown.
// Normalize it so components can always read `kind` and `message`.
export function toAppError(err: unknown): AppError {
  if (typeof err === "object" && err !== null && "kind" in err && "message" in err) {
    return err as AppError;
  }
  return { kind: "unknown", message: String(err) };
}
