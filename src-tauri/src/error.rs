// The single error type returned by every command.
// It serializes to `{ "kind": "...", "message": "..." }` so the frontend can
// branch on `kind` and show `message` to the user.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ErrorKind {
    GitNotFound,
    PathNotFound,
    NotARepository,
    GitFailed,
    Io,
    RepoAlreadyAdded,
    Storage,
    InvalidInput,
    NotFound,
}

#[derive(Debug, Serialize)]
pub struct AppError {
    pub kind: ErrorKind,
    pub message: String,
}

impl AppError {
    pub fn new(kind: ErrorKind, message: String) -> AppError {
        AppError { kind, message }
    }
}
