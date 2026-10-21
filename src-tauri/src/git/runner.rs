use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Output, Stdio};

use crate::error::{AppError, ErrorKind};

/// Runs `git <args>` inside `cwd` and returns its stdout.
///
/// Arguments are passed as a list, never through a shell, so paths with
/// spaces or special characters are safe.
pub fn run_git(cwd: &Path, args: &[&str]) -> Result<String, AppError> {
    let stdout = run_git_bytes(cwd, args)?;
    // Lossy: a file name in a non-UTF-8 encoding becomes "�" instead of an error.
    Ok(String::from_utf8_lossy(&stdout).to_string())
}

/// Like `run_git`, but returns stdout as raw bytes. Needed for file contents,
/// which may be binary or use an encoding other than UTF-8.
pub fn run_git_bytes(cwd: &Path, args: &[&str]) -> Result<Vec<u8>, AppError> {
    let output = git_command(cwd, args)?.output().map_err(spawn_error)?;
    check_output(output, args)
}

/// Like `run_git`, but also writes `input` to git's stdin, e.g. a commit
/// message for `git commit -F -`. Text sent this way can never be mistaken
/// for an option, and has no command-line length limit.
pub fn run_git_with_input(cwd: &Path, args: &[&str], input: &str) -> Result<String, AppError> {
    let mut child = git_command(cwd, args)?
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(spawn_error)?;

    // take(): move the pipe out of `child`; dropping it at the end of this
    // block closes stdin, which tells git the input is complete.
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes()).map_err(|err| {
            AppError::new(ErrorKind::Io, format!("Could not write to git: {err}"))
        })?;
    }
    let output = child.wait_with_output().map_err(spawn_error)?;
    let stdout = check_output(output, args)?;
    Ok(String::from_utf8_lossy(&stdout).to_string())
}

// Turns a non-zero exit code into an error carrying git's own message.
fn check_output(output: Output, args: &[&str]) -> Result<Vec<u8>, AppError> {
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let message = if stderr.is_empty() {
            format!("git {} failed ({})", args.join(" "), output.status)
        } else {
            stderr
        };
        return Err(AppError::new(ErrorKind::GitFailed, message));
    }
    Ok(output.stdout)
}

/// Builds the `git` process with the settings every git call needs.
/// Shared by `run_git` and the long-running commands in operations.rs.
pub(crate) fn git_command(cwd: &Path, args: &[&str]) -> Result<Command, AppError> {
    // A missing working directory makes spawn fail with NotFound, which would
    // otherwise be reported as "git not installed".
    if !cwd.is_dir() {
        return Err(AppError::new(
            ErrorKind::PathNotFound,
            format!("Folder does not exist: {}", cwd.display()),
        ));
    }

    let mut command = Command::new("git");
    command
        .args(args)
        .current_dir(cwd)
        // Fail instead of hanging when git wants a username/password.
        .env("GIT_TERMINAL_PROMPT", "0")
        // Keep git messages in English so we can recognise them in stderr.
        .env("LC_ALL", "C");
    Ok(command)
}

pub(crate) fn spawn_error(err: io::Error) -> AppError {
    if err.kind() == io::ErrorKind::NotFound {
        AppError::new(
            ErrorKind::GitNotFound,
            "git was not found. Install git and make sure it is on PATH.".to_string(),
        )
    } else {
        AppError::new(ErrorKind::Io, format!("Could not start git: {err}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn returns_stdout_on_success() {
        let dir = TempDir::new().unwrap();
        let out = run_git(dir.path(), &["--version"]).unwrap();
        assert!(out.starts_with("git version"), "unexpected output: {out}");
    }

    #[test]
    fn returns_stderr_when_git_fails() {
        let dir = TempDir::new().unwrap();
        let err = run_git(dir.path(), &["no-such-command"]).unwrap_err();
        assert_eq!(err.kind, ErrorKind::GitFailed);
        assert!(
            err.message.contains("no-such-command"),
            "message: {}",
            err.message
        );
    }

    #[test]
    fn passes_input_on_stdin() {
        let dir = TempDir::new().unwrap();
        // hash-object --stdin prints the id of whatever it reads.
        let out = run_git_with_input(dir.path(), &["hash-object", "--stdin"], "hello\n").unwrap();
        // The well-known id of the blob "hello\n".
        assert_eq!(out.trim(), "ce013625030ba8dba906f756967f9e9ca394464a");
    }

    #[test]
    fn reports_missing_folder() {
        let dir = TempDir::new().unwrap();
        let missing = dir.path().join("does-not-exist");
        let err = run_git(&missing, &["status"]).unwrap_err();
        assert_eq!(err.kind, ErrorKind::PathNotFound);
    }
}
