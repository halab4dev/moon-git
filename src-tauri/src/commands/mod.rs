// Tauri commands, grouped by feature. Commands stay thin: validate input,
// call into git/ or storage/, and return a model.

pub mod clone;
pub mod commit;
pub mod diff;
pub mod fetch;
pub mod group;
pub mod log;
pub mod pull;
pub mod push;
pub mod repo;
pub mod stage;
pub mod status;

use tauri::{AppHandle, Emitter};

use crate::error::{AppError, ErrorKind};
use crate::git::operations::parse_percent;
use crate::models::GitProgress;

/// Event name the frontend listens to for progress of any long git command.
pub const PROGRESS_EVENT: &str = "git-progress";

/// Sends one line of git output to the frontend as a "git-progress" event,
/// tagged with the operation id so each listener can pick its own lines.
pub fn emit_progress(app: &AppHandle, op_id: &str, line: &str) {
    let (phase, percent) = match parse_percent(line) {
        Some((phase, percent)) => (Some(phase), Some(percent)),
        None => (None, None),
    };
    let event = GitProgress {
        op_id: op_id.to_string(),
        line: line.to_string(),
        phase,
        percent,
    };
    if let Err(err) = app.emit(PROGRESS_EVENT, event) {
        eprintln!("Could not send progress: {err}");
    }
}

/// Runs blocking work (usually a git process) on a worker thread and waits
/// for it without blocking the UI thread. Use it from `async` commands.
///
/// `Send + 'static`: the work moves to another thread, so it may only
/// capture values it owns (no borrowed references) that are safe to send.
pub async fn run_blocking<T>(
    work: impl FnOnce() -> Result<T, AppError> + Send + 'static,
) -> Result<T, AppError>
where
    T: Send + 'static,
{
    match tauri::async_runtime::spawn_blocking(work).await {
        Ok(result) => result,
        // Only happens if the work panicked.
        Err(err) => Err(AppError::new(
            ErrorKind::Io,
            format!("A background task stopped unexpectedly: {err}"),
        )),
    }
}
