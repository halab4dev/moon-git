# MoonGit

A desktop Git client for macOS and Linux, built as a personal replacement for SmartGit.

MoonGit is a thin GUI over the **git command line** installed on your machine. It doesn't use libgit2: every action runs a real `git` command, so the results are exactly what git itself would do.

> **Vibe-coded.** MoonGit is a vibe-coding project: almost all of the code was written by an AI assistant (Claude, via Claude Code), step by step, with a human choosing the features, testing each step and reviewing the result. Keep that in mind before trusting it with important repositories, and keep backups / remotes as usual.

**Stack:** [Tauri v2](https://tauri.app) · Rust backend · Vue 3 + TypeScript + Pinia frontend · Monaco editor for diffs and conflicts.

## Features

### Repositories
- Keep a list of local repositories, organized in **groups and subgroups** (create, rename, delete, move repos between groups).
- Add an existing local repository, or **clone** a remote one with live progress and a Cancel button.

### Working tree and commits
- Changed files, split into staged / unstaged / untracked, with SmartGit-style states (Modified, Added, Missing, Conflicted…).
- Stage / unstage files, and **discard** changes. Discarded new files go to the system Trash, not straight to deletion.
- **Side-by-side diff** (Monaco), opened in its own window: working tree, staged, or conflict view.
- **Commit** with a message, or **amend** the last commit.

### Sync
- **Fetch** (all remotes), **pull** and **push**, with live progress and a Cancel button.
- Ahead / behind counts for the current branch.
- When a branch has diverged, pull asks whether to rebase or merge.
- Push never forces.

### History
- **Commit graph** with lanes. Choose which local branches, remote branches and tags to show.
- Filter by commit id prefix or by message, in two modes: **highlight** (dim the commits that don't match) or **hide** them.
- A virtualized list that loads more commits as you scroll, to stay smooth on large repositories.
- Click a commit to see its details, its changed files and their diffs.

### Branches and remotes
- **Branches panel** in the sidebar, like SmartGit's:
  - local branches show ✓ for the current one, ↑ahead ↓behind, and "gone" when the upstream was deleted on the server;
  - remote branches are grouped by remote, each with its URL (passwords or tokens in a URL are masked);
  - sort A→Z / Z→A, and collapse groups.
- Branch actions: check out (double-click), create, rename, delete local branches, delete a branch on the server, set or unset the upstream branch.
- **Add a remote**, and edit a remote's URL.

### Advanced operations
- **Stash:** create (with a message, optionally including untracked files), list, apply, pop, drop.
- **Merge:** fast-forward, create a merge commit, or only bring the changes into the working tree.
- **Rebase** (non-interactive) onto a chosen branch.
- **Reset** the current branch to a commit: soft, mixed or hard. Hard asks twice.
- **Cherry-pick:** as a new commit, or only apply the changes to the working tree.
- **Conflicts:**
  - Merges, rebases and cherry-picks in progress are detected, with Continue / Skip / Abort.
  - Conflicted files are listed. Take ours or theirs for a whole file.
  - A **3-column Conflict Solver** window: your side, an editable result and their side. Jump between conflicts, take left / right / both, then *Save & Mark Resolved*.

### Safety
- Destructive actions (hard reset, deleting branches, discarding changes, aborting operations…) always ask for confirmation first.
- git is always called with argument arrays, never through a shell string.
- `GIT_TERMINAL_PROMPT=0` is set, so git never hangs waiting for a password.
- git's error messages are turned into readable messages, never swallowed.

### Not done yet
- **Account management**:
  - per-server HTTPS tokens stored in the OS keychain;
  - per-repository `user.name` / `user.email`;
  - choosing an SSH key per server.

  Until then, MoonGit uses whatever your system git already uses: credential helpers, SSH agent and keys, and your git config.
- **Windows:** not supported yet.
  - The code tries to stay OS-independent, but it's untested there.
  - Cancelling a long operation only stops the `git` process itself.

## Requirements

| Tool | Version | Notes |
|---|---|---|
| git | 2.33 or newer | Must be on `PATH`; MoonGit runs it for everything |
| Node.js | 20.19+ or 22.12+ | Required by Vite |
| pnpm | 9+ | Package manager used by the project (`npm i -g pnpm` or `corepack enable`) |
| Rust | stable | Install with [rustup](https://rustup.rs) |

Each OS also needs some system packages:

### macOS
Install the Xcode Command Line Tools; the full Xcode app is not needed:

```sh
xcode-select --install
```

If the build fails with linker / `xcrun` errors, point `xcode-select` at the Command Line Tools:

```sh
sudo xcode-select --switch /Library/Developer/CommandLineTools
```

Then open a new terminal.

### Linux (Debian / Ubuntu)

```sh
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

For other distributions, see the [Tauri prerequisites](https://tauri.app/start/prerequisites/).

### Windows (not supported yet)
To try it anyway, you need:
- Microsoft C++ Build Tools ("Desktop development with C++");
- WebView2, which is preinstalled on Windows 10 (1803+) and 11.

## Run locally

```sh
pnpm install
pnpm tauri dev
```

This starts the Vite dev server on port 1420 and opens the app window.
- Frontend changes reload instantly.
- Rust changes trigger a rebuild and restart of the app.
- The first run compiles all Rust dependencies, which takes a few minutes.

`pnpm dev` alone only starts the web frontend in a browser, without the Rust backend, so git actions won't work there. Use `pnpm tauri dev`.

## Tests

```sh
# Rust: git command parsing and operations, run against temporary repositories
cd src-tauri && cargo test

# TypeScript: graph lane layout, diff and conflict helpers…
pnpm test

# Type-check and build the frontend
pnpm build
```

## Build the app

```sh
pnpm install
pnpm tauri build
```

- Tauri builds for the OS you are on. Cross-compiling, e.g. a Linux package on a Mac, isn't supported: build on each OS, or use CI runners for each one.
- Installers end up in `src-tauri/target/release/bundle/`:

| OS | Output |
|---|---|
| macOS | `macos/MoonGit.app`, `dmg/MoonGit_<version>_<arch>.dmg` |
| Linux | `deb/*.deb`, `rpm/*.rpm`, `appimage/*.AppImage` |
| Windows | `msi/*.msi`, `nsis/*-setup.exe` |

To build only some formats, pass `--bundles`:

```sh
pnpm tauri build --bundles dmg        # macOS
pnpm tauri build --bundles deb,appimage   # Linux
```

### macOS: Apple Silicon + Intel in one app

```sh
rustup target add aarch64-apple-darwin x86_64-apple-darwin
pnpm tauri build --target universal-apple-darwin
```

The output goes to `src-tauri/target/universal-apple-darwin/release/bundle/`.

### macOS: unsigned builds
The app is not signed or notarized. If macOS refuses to open it ("damaged" / "unidentified developer"), right-click → **Open**, or remove the quarantine flag:

```sh
xattr -cr /Applications/MoonGit.app
```

## Project structure

```
src-tauri/src/
  lib.rs          registers the Tauri commands
  commands/       thin #[tauri::command] functions
  git/            runs git and parses its output (one file per area: status, log, branch, merge…)
  models.rs       data sent to the frontend
  error.rs        the error type shared by every command
  storage/        app settings: repository list and groups (no secrets)
src/
  api/            typed wrappers around invoke(), one per command
  types/          TypeScript types matching models.rs
  stores/         Pinia stores
  composables/    reusable Vue logic (e.g. resizable split panes)
  components/     Vue components (branch panel, dialogs, diff, conflict solver…)
  views/          repository, history and stash views; diff and conflict windows
  graph/          commit graph lane layout
  conflict/       line diff and conflict block helpers
docs/             development notes and progress
```

## Contributing

Bug reports, ideas and pull requests are welcome.

### Reporting a bug
Open an [issue](https://github.com/halab4dev/moon-git/issues) with:
- your OS and version, and `git --version`;
- what you did, what you expected, and what happened instead;
- the exact error message shown by MoonGit, if there was one.

Before pasting logs or screenshots, remove any tokens, passwords or private repository URLs.

### Pull requests
1. For anything bigger than a small fix, open an issue first, so we can agree on the approach.
2. Keep each pull request to one feature or fix.
3. Follow the existing rules of the code base:
   - git is always run with an **argument array**, never through a shell string;
   - prefer machine-readable git output (`--porcelain`, `-z`, `--format` with explicit separators);
   - every destructive action needs a confirmation in the UI;
   - git errors become readable messages, never swallowed;
   - no secrets in files, logs or the console;
   - **no libgit2 / `git2-rs`**: MoonGit stays a thin layer over the git CLI.
4. New git parsing or git operations come with **Rust tests** that run against a temporary repository (see `src-tauri/src/git/test_support.rs`). Frontend logic that isn't UI (like the graph layout) gets Vitest tests.
5. Before opening the pull request, make sure these pass:

   ```sh
   cd src-tauri && cargo fmt --check && cargo clippy --all-targets && cargo test
   cd .. && pnpm test && pnpm build
   ```

## Acknowledgments

- [SmartGit](https://www.syntevo.com/smartgit/), the inspiration for MoonGit's layout and workflow (branches panel, conflict solver, merge options). MoonGit is not affiliated with syntevo.
- [Tauri](https://tauri.app), for a small, fast desktop shell with a Rust backend.
- [Vue](https://vuejs.org), [Pinia](https://pinia.vuejs.org) and [Vite](https://vite.dev), for the frontend.
- [Monaco Editor](https://microsoft.github.io/monaco-editor/), which powers the diff viewer and the conflict solver.
- [`trash`](https://crates.io/crates/trash), for moving discarded files to the system Trash.
- [Git](https://git-scm.com) itself: MoonGit only puts a window around it.
- [Claude Code](https://claude.com/claude-code), which wrote most of the code.

## License

[MIT](LICENSE) © 2026 halab4dev
