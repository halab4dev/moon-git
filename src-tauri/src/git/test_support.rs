// Helpers shared by the tests of fetch, pull and push: a bare "server"
// repository in a temp folder, and clones of it acting as users' machines.

use std::fs;
use std::path::Path;

use tempfile::TempDir;

use crate::git::runner::run_git;

/// Runs git and panics on failure; tests want to stop at the first problem.
pub fn git(dir: &Path, args: &[&str]) -> String {
    run_git(dir, args).unwrap()
}

/// Writes `name` (content = its name) and commits it.
pub fn commit_file(dir: &Path, name: &str) {
    fs::write(dir.join(name), format!("{name}\n")).unwrap();
    git(dir, &["add", "--", name]);
    git(dir, &["commit", "--quiet", "-m", name]);
}

/// A clone of `server`, with an identity set in the clone itself so that
/// commits, merges and rebases work on machines without a global user.name.
pub fn clone_of(server: &Path) -> TempDir {
    let work = TempDir::new().unwrap();
    let url = server.to_string_lossy().to_string();
    git(work.path(), &["clone", "--quiet", &url, "."]);
    git(work.path(), &["config", "user.name", "Test"]);
    git(work.path(), &["config", "user.email", "test@example.com"]);
    work
}

/// A bare "server" repo with one commit on main, and a clone tracking origin/main.
pub fn server_and_clone() -> (TempDir, TempDir) {
    let server = TempDir::new().unwrap();
    git(
        server.path(),
        &["init", "--quiet", "--bare", "--initial-branch=main"],
    );
    let work = clone_of(server.path());
    // An empty clone has no branch yet; make sure the first one is "main".
    git(work.path(), &["symbolic-ref", "HEAD", "refs/heads/main"]);
    commit_file(work.path(), "first.txt");
    git(work.path(), &["push", "--quiet", "-u", "origin", "main"]);
    (server, work)
}
