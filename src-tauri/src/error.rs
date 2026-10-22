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
    /// The user cancelled a long-running operation.
    Cancelled,
    /// The server wants a login and none could be provided.
    AuthRequired,
    /// git printed something our parser does not understand.
    UnexpectedOutput,
    /// git does not know the author name/email to put on a commit.
    IdentityUnknown,
    /// A merge or rebase stopped because files have conflicts.
    Conflict,
    /// The server refused a push because it has commits we don't have.
    PushRejected,
    /// A branch to delete has commits that no other branch contains.
    NotMerged,
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
