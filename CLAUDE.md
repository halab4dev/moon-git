# MoonGit — Git GUI client (Tauri + Rust + Vue/TypeScript)

## Goal
Build a desktop Git client to replace SmartGit for personal use, running on macOS and Ubuntu (Windows support later, so avoid OS-dependent code from the start).

## About the user — VERY IMPORTANT
- I know **Java** best and can code in **NodeJS**. **Rust, Vue and Tauri are new** to me.
- I don't only want the app to work, I want to **understand what the code does**. For Rust and Vue, a basic level of understanding is enough.
- So work in **learning mode**:
    1. Work in small steps, each step only one feature or one clear part.
    2. After each step, explain **in Vietnamese**: which files were created/changed, why, and how the data flows (Vue → invoke → Rust → git → back).
    3. When a Rust concept appears for the first time (ownership, borrowing, `Result`, `?`, `Option`, trait, `async`, lifetime…), explain it briefly and **compare it with Java** when possible.
    4. When a Vue concept appears for the first time (`ref`, `computed`, `watch`, props/emit, store…), explain it briefly and compare it with a familiar way of doing things if there is one.
    5. Don't add a new library without saying what it is for and why it was chosen.
    6. At the end of each step: tell me how to run and check it, then **stop and wait for my confirmation** before moving on to the next step.
- Prefer readable code over "clever" code. Avoid complex macros, nested generics or hard-to-follow tricks when they aren't really needed.
- Code comments are written in English, short, explaining "why" where it isn't obvious.

## Technology
- **Tauri v2**
- **Backend:** Rust (stable). Use `serde` to pass data to the frontend.
- **Frontend:** Vue 3 (Composition API, `<script setup lang="ts">`), TypeScript strict mode, Vite, Pinia for state.
- **Git operations:** call the **git CLI** through `std::process::Command` (or `tokio::process` for long operations). **Don't** use `git2-rs`/libgit2, to keep the Rust part thin and easy to understand, and the behavior identical to the system's git.
- **Diff viewer:** Monaco diff editor (or a lighter option if there is a reason, but it must be explained).
- **Storing secrets:** the `keyring` crate (macOS Keychain, Secret Service on Ubuntu, Windows Credential Manager).

## Architecture
```
src-tauri/src/
  main.rs / lib.rs     # registers the Tauri commands
  commands/            # #[tauri::command] functions, thin, only call down into git/
  git/                 # runs git commands + parses output, one file per area (status.rs, log.rs, branch.rs…)
  models.rs            # data structs returned to the frontend (derive Serialize)
  error.rs             # shared error type, serializable to the frontend
  storage/             # app settings: repository list, groups, accounts (no secrets)
src/
  api/                 # typed wrappers for invoke(), one TS function per command
  types/               # TS interfaces matching models.rs
  stores/              # Pinia stores
  components/          # Vue components
  views/
```

## Rules for working with git
- Always pass arguments as an **args array**, **never** build a shell command string (avoids bugs with paths containing spaces, and injection).
- Use machine-readable output formats: `git status --porcelain=v2 -z`, `git log` with `--format` and explicit separators (e.g. `%x1f`, `%x1e`), `-z` when possible.
- Set `GIT_TERMINAL_PROMPT=0` so git never hangs waiting for a password.
- Long operations (clone, fetch, push, pull) run asynchronously, send **progress** to the UI through Tauri events, and can be cancelled.
- Errors from git (stderr, exit code) must be turned into understandable messages in the UI, never swallowed.
- Parsers of git output must have **Rust unit tests**, using temporary repositories created in the test.

## Features and roadmap

### Phase 1 — usable every day
1. A working Tauri + Vue + TS project skeleton, basic layout (repository sidebar | main content).
2. Add a local repository (choose a folder, check it is a git repository) and **clone** a remote repository (with progress).
3. Repository **groups / subgroups**: a tree in the sidebar, create/rename/delete groups, move repositories between groups (drag and drop can come later). Stored as JSON in the app data folder.
4. List of changed files (staged / unstaged / untracked), stage/unstage files.
5. View the diff of each file (side-by-side).
6. Commit (message, amend).
7. Fetch, pull, push; show ahead/behind for the current branch.
8. **Account management:**
    - Accounts per server (GitHub, GitLab, self-hosted servers…): HTTPS tokens stored in the keyring, secrets **never** stored as text.
    - Attach an account to each repository, and a separate `user.name` / `user.email` per repository.
    - SSH: use the system's ssh; allow choosing a key per server/repository.
    - Provide credentials to git safely (e.g. through `GIT_ASKPASS` pointing back to the app, or a credential helper). Propose an approach and explain it before implementing.

### Phase 2 — history and branches
9. **Commit history graph:**
    - Choose which branches (local/remote/tag) to show.
    - Filter by **commit id** (hash prefix) and by **commit message**.
    - Two filter modes: **highlight** (keep the graph, dim the commits that don't match; default) and **hide** the commits that don't match.
    - The lane layout algorithm is written in TypeScript, as a separate module with tests, with an explanation of how it works.
    - A virtualized list (only renders what is visible) with incremental loading, to stay smooth on repositories with tens of thousands of commits, including on WebKitGTK (Ubuntu).
    - Click a commit: see its details, its list of files and their diffs.
10. **Branches:** create, checkout, rename, delete (local/remote), tracking branch.

### Phase 3 — advanced operations
11. **Stash:** create (with a message, optionally including untracked files), list, apply, pop, drop.
12. **Merge:** two options: create a merge commit (`--no-ff`), or only bring the changes into the working tree (`--squash` / `--no-commit`).
13. **Rebase** (non-interactive) onto a chosen branch.
14. **Cherry-pick:** two options: keep it as a commit, or only copy the changes into the working tree (`-n`).
15. **Conflict handling** (required together with 11–14): detect an in-progress merge/rebase/cherry-pick, list the conflicted files, open an external compare tool or choose ours/theirs, mark as resolved, Continue / Abort buttons.

## Things to avoid
- Don't do several features at once in one step.
- Don't do a big refactor on your own without asking first.
- Don't write tokens/passwords to files, logs or the console.
- Don't run destructive git commands (`reset --hard`, `push --force`, `clean`, deleting branches…) without a confirmation dialog in the UI.
