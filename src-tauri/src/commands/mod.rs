// Tauri commands, grouped by feature. Commands stay thin: validate input,
// call into git/ or storage/, and return a model.

pub mod clone;
pub mod group;
pub mod repo;
pub mod status;

use crate::error::{AppError, ErrorKind};

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
