// Long-running git commands (clone now; fetch, pull, push later) that report
// progress while they run and can be cancelled from the UI.

use std::collections::HashMap;
use std::io::{self, Read};
use std::path::Path;
use std::process::{Child, ExitStatus, Stdio};
use std::sync::{Mutex, MutexGuard};

use crate::error::{AppError, ErrorKind};
use crate::git::runner::{git_command, spawn_error};

/// Running git processes, keyed by an operation id chosen by the frontend.
/// Tauri keeps one instance; `cancel` can be called from another command
/// while `run_git_with_progress` is still reading output.
pub struct Operations {
    running: Mutex<HashMap<String, RunningOp>>,
}

struct RunningOp {
    child: Child,
    cancelled: bool,
}

impl Operations {
    pub fn new() -> Operations {
        Operations {
            running: Mutex::new(HashMap::new()),
        }
    }

    /// Stops the operation. Unknown ids are ignored: the operation already finished.
    pub fn cancel(&self, op_id: &str) -> Result<(), AppError> {
        let mut running = self.lock()?;
        let Some(op) = running.get_mut(op_id) else {
            return Ok(());
        };
        op.cancelled = true;

        if let Err(err) = terminate(&mut op.child) {
            // Failing to stop a process that has already exited is fine.
            if let Ok(None) = op.child.try_wait() {
                return Err(AppError::new(
                    ErrorKind::Io,
                    format!("Could not stop git: {err}"),
                ));
            }
        }
        Ok(())
    }

    fn start(&self, op_id: &str, child: Child) -> Result<(), AppError> {
        let mut running = self.lock()?;
        if running.contains_key(op_id) {
            return Err(AppError::new(
                ErrorKind::InvalidInput,
                format!("Operation {op_id} is already running."),
            ));
        }
        running.insert(
            op_id.to_string(),
            RunningOp {
                child,
                cancelled: false,
            },
        );
        Ok(())
    }

    /// Removes the operation and waits for git to exit. Returns (exit status, was cancelled).
    fn finish(&self, op_id: &str) -> Result<(ExitStatus, bool), AppError> {
        // Take it out of the map first, so waiting doesn't hold the lock.
        let op = self.lock()?.remove(op_id);
        let Some(mut op) = op else {
            return Err(AppError::new(
                ErrorKind::Io,
                format!("Operation {op_id} disappeared."),
            ));
        };
        let status = op.child.wait().map_err(|err| {
            AppError::new(ErrorKind::Io, format!("Could not wait for git: {err}"))
        })?;
        Ok((status, op.cancelled))
    }

    fn lock(&self) -> Result<MutexGuard<'_, HashMap<String, RunningOp>>, AppError> {
        self.running.lock().map_err(|_| {
            AppError::new(
                ErrorKind::Io,
                "Operation list is in a bad state. Please restart the app.".to_string(),
            )
        })
    }
}

/// Runs `git <args>` in `cwd`, calling `on_line` for every line git prints to
/// stderr (where git writes progress). Blocks until git exits, so call it from
/// a background thread.
pub fn run_git_with_progress(
    ops: &Operations,
    op_id: &str,
    cwd: &Path,
    args: &[&str],
    mut on_line: impl FnMut(&str),
) -> Result<(), AppError> {
    let mut command = git_command(cwd, args)?;
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    use_own_process_group(&mut command);

    let mut child = command.spawn().map_err(spawn_error)?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::new(ErrorKind::Io, "git has no stderr pipe.".to_string()))?;
    ops.start(op_id, child)?;

    // Keep non-progress lines: if git fails, the reason is among them.
    let mut messages: Vec<String> = Vec::new();
    read_lines(stderr, |line| {
        on_line(line);
        if parse_percent(line).is_none() {
            messages.push(line.to_string());
        }
    });

    // stderr closed: git has exited (or was stopped). Collect the result.
    let (status, cancelled) = ops.finish(op_id)?;
    if cancelled {
        return Err(AppError::new(
            ErrorKind::Cancelled,
            "Cancelled.".to_string(),
        ));
    }
    if !status.success() {
        return Err(failure(&messages, args, status));
    }
    Ok(())
}

/// Splits git's stderr into lines. Progress updates end with '\r' (rewrite the
/// same terminal line), normal messages with '\n'; both count as a line here.
fn read_lines(mut input: impl Read, mut on_line: impl FnMut(&str)) {
    let mut buffer = [0u8; 4096];
    let mut current: Vec<u8> = Vec::new();

    loop {
        let count = match input.read(&mut buffer) {
            Ok(0) => break, // end of stream
            Ok(count) => count,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            // A broken pipe just ends the stream; the exit status reports the failure.
            Err(_) => break,
        };
        for &byte in &buffer[..count] {
            if byte == b'\r' || byte == b'\n' {
                emit_line(&mut current, &mut on_line);
            } else {
                current.push(byte);
            }
        }
    }
    emit_line(&mut current, &mut on_line);
}

fn emit_line(current: &mut Vec<u8>, on_line: &mut impl FnMut(&str)) {
    if !current.is_empty() {
        let line = String::from_utf8_lossy(current);
        on_line(line.trim_end());
        current.clear();
    }
}

/// Reads "Receiving objects:  45% (450/1000), 1.20 MiB | 1.00 MiB/s"
/// as ("Receiving objects", 45). Lines without a percentage give None.
pub fn parse_percent(line: &str) -> Option<(String, u8)> {
    let line = line.strip_prefix("remote: ").unwrap_or(line);
    let (phase, rest) = line.split_once(':')?;
    let number = rest.trim_start().split('%').next()?;
    // A '%' must actually follow the number, otherwise "x: 5, done." would match.
    if !rest.contains('%') {
        return None;
    }
    let percent: u8 = number.trim().parse().ok()?;
    if percent > 100 {
        return None;
    }
    Some((phase.trim().to_string(), percent))
}

/// Turns git's messages into one error for the UI.
fn failure(messages: &[String], args: &[&str], status: ExitStatus) -> AppError {
    let all = messages.join("\n");
    if all.contains("terminal prompts disabled") || all.contains("Authentication failed") {
        return AppError::new(
            ErrorKind::AuthRequired,
            "The server asked for a username and password. Logging in from MoonGit is not supported yet; \
             use an SSH URL or a git credential helper for now."
                .to_string(),
        );
    }

    // " ! [rejected]  main -> main (fetch first)" or "(non-fast-forward)".
    let rejected = messages.iter().any(|m| {
        m.contains("[rejected]") && (m.contains("fetch first") || m.contains("non-fast-forward"))
    });
    if rejected {
        return AppError::new(
            ErrorKind::PushRejected,
            "The server has commits that you don't have yet. Pull first, then push again."
                .to_string(),
        );
    }

    // "fatal:" / "error:" lines carry the reason; fall back to the last thing git said.
    let important: Vec<&str> = messages
        .iter()
        .map(|m| m.as_str())
        .filter(|m| m.starts_with("fatal:") || m.starts_with("error:"))
        .collect();
    let message = if !important.is_empty() {
        important.join("\n")
    } else if let Some(last) = messages.last() {
        last.clone()
    } else {
        format!("git {} failed ({status})", args.first().unwrap_or(&""))
    };
    AppError::new(ErrorKind::GitFailed, message)
}

// git starts helper processes (git-remote-https, ssh, index-pack). Putting
// them in their own process group lets `terminate` stop all of them at once;
// otherwise a helper stuck on a slow network could outlive the cancel.
#[cfg(unix)]
fn use_own_process_group(command: &mut std::process::Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0); // 0 = new group whose id is the child's pid
}

#[cfg(not(unix))]
fn use_own_process_group(_command: &mut std::process::Command) {
    // Windows: not handled yet; `terminate` stops only git itself.
}

// SIGTERM (not SIGKILL) gives git the chance to delete a half-finished clone.
// SIGCONT afterwards: a process that tried to read from the terminal (e.g. ssh
// asking about an unknown host key when the app was started from a terminal)
// is stopped by the OS, and a stopped process only acts on SIGTERM once resumed.
#[cfg(unix)]
fn terminate(child: &mut Child) -> io::Result<()> {
    use rustix::process::{Pid, Signal, kill_process_group};
    let group = Pid::from_child(child);
    kill_process_group(group, Signal::TERM)?;
    kill_process_group(group, Signal::CONT)?;
    Ok(())
}

#[cfg(not(unix))]
fn terminate(child: &mut Child) -> io::Result<()> {
    child.kill()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    use tempfile::TempDir;

    fn lines_of(input: &[u8]) -> Vec<String> {
        let mut lines = Vec::new();
        read_lines(input, |line| lines.push(line.to_string()));
        lines
    }

    #[test]
    fn splits_on_carriage_return_and_newline() {
        let input = b"Cloning into 'x'...\nReceiving objects:  10% (1/10)\rReceiving objects: 100% (10/10), done.\n";
        assert_eq!(
            lines_of(input),
            vec![
                "Cloning into 'x'...",
                "Receiving objects:  10% (1/10)",
                "Receiving objects: 100% (10/10), done.",
            ]
        );
    }

    #[test]
    fn keeps_last_line_without_newline_and_skips_empty_lines() {
        assert_eq!(lines_of(b"\r\n\nfatal: oops"), vec!["fatal: oops"]);
    }

    #[test]
    fn parses_progress_lines() {
        assert_eq!(
            parse_percent("Receiving objects:  45% (450/1000), 1.20 MiB | 1.00 MiB/s"),
            Some(("Receiving objects".to_string(), 45))
        );
        assert_eq!(
            parse_percent("remote: Counting objects: 100% (5/5), done."),
            Some(("Counting objects".to_string(), 100))
        );
    }

    #[test]
    fn ignores_lines_without_percent() {
        assert_eq!(parse_percent("Cloning into 'a:b'..."), None);
        assert_eq!(parse_percent("remote: Enumerating objects: 5, done."), None);
        assert_eq!(parse_percent("fatal: repository not found"), None);
    }

    #[test]
    fn reports_fatal_line_on_failure() {
        let dir = TempDir::new().unwrap();
        let ops = Operations::new();
        let err = run_git_with_progress(
            &ops,
            "op",
            dir.path(),
            &["clone", "--", "/no/such/repo", "x"],
            |_| {},
        )
        .unwrap_err();
        assert_eq!(err.kind, ErrorKind::GitFailed);
        assert!(
            err.message.starts_with("fatal:"),
            "message: {}",
            err.message
        );
    }

    // A git alias that runs `sleep` gives a slow command with a child process,
    // like a clone stuck on the network.
    #[cfg(unix)]
    #[test]
    fn cancel_stops_git_and_its_children() {
        let dir = TempDir::new().unwrap();
        let ops = Operations::new();
        let started = Instant::now();

        // Scoped threads may borrow `ops` and `dir` because they end before the scope does.
        let result = std::thread::scope(|scope| {
            let worker = scope.spawn(|| {
                run_git_with_progress(
                    &ops,
                    "slow",
                    dir.path(),
                    &["-c", "alias.wait=!sleep 30", "wait"],
                    |_| {},
                )
            });
            std::thread::sleep(Duration::from_millis(300));
            ops.cancel("slow").unwrap();
            worker.join().unwrap()
        });

        assert_eq!(result.unwrap_err().kind, ErrorKind::Cancelled);
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "cancel took {:?}",
            started.elapsed()
        );
    }

    #[test]
    fn cancel_of_unknown_operation_is_ignored() {
        assert!(Operations::new().cancel("nothing").is_ok());
    }
}
