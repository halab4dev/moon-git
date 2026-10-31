# MoonGit — Plan and progress

Last updated: 2026-10-27

**Current state:** Phase 3 is done, plus steps 25–30 added on request. Steps 31a (menu bar) and 31b (Branch → Checkout… dialog) are done. Step 32 (welcome picture) is done. Step 33 (dark mode) is done. Accounts resumed (2026-10-28): step 12 (MoonGit as git's credential helper) is done; 13a (saved accounts: accounts.json + keyring) is done; 13b (Sign in dialog) and 13c (MoonGit → Accounts…) are done (not yet tried against a real HTTPS server). Step 34 (drag repositories into / out of groups) is done. Step 14 (commit identity per repository and per group) is done. Step 15 (SSH key and HTTPS login, per repository and per group) is done: every item of the CLAUDE.md roadmap is now done. Steps 35a (drag whole groups) and 35b (select several repositories: drag, Move To, Commit Identity, Login, Remove) are done.

Legend: ✅ done · 🟡 in progress / waiting for confirmation · ⏸ on hold · ⬜ not started

Detailed notes for each step (in Vietnamese): [docs/learning/](learning/README.md)

---

## Phase 1 — usable every day

| # | Step | Status | Notes |
|---|---|---|---|
| 0 | Set up the environment | ✅ | Rust 1.99.1, Node 24, pnpm 10, git 2.33, Command Line Tools (SDK 13.1); `xcode-select` points to `/Library/Developer/CommandLineTools` |
| 1 | Tauri v2 + Vue 3 + TS skeleton, sidebar \| content layout, `greet` command | ✅ | UI confirmed (2026-10-21) |
| 2 | Layer that runs git + shared error type | ✅ | `git/runner.rs`, `error.rs`, `check_repo` command. 7 tests pass, clippy clean. 4 cases confirmed in the UI (2026-10-21) |
| 3 | Repository list + add a local repository | ✅ | `storage/repos.rs` (`repos.json`, written via a temp file + rename), dialog plugin, Pinia store. 13 tests pass, clippy clean. Confirmed in the UI (2026-10-21) |
| 4 | Repository groups / subgroups | ✅ | `storage/repo_list.rs`, `repos.json` v2, native context menus, `NameDialog`. 23 tests pass. Confirmed (2026-10-21), SVG folder icon added. Drag and drop of repositories: step 34; of groups: step 35a |
| 5 | Clone with progress + cancel | ✅ | `git/operations.rs` (worker thread + Tauri event + cancelling the whole process group), `git/clone.rs`, `CloneDialog`. 36 tests pass. Confirmed (2026-10-21). Fetch/pull/push reuse `operations.rs` |
| 6 | List of changed files | ✅ | `git/status.rs` (porcelain v2 -z, `--no-optional-locks`), `FileTable.vue` table (Name/State/Location, sortable columns, like SmartGit), refreshes on focus. 45 tests pass. Confirmed (2026-10-21) |
| 7 | Stage / unstage | ✅ | `git/stage.rs` (`--literal-pathspecs`, batches of 100, repositories without commits), desktop-style row selection, buttons + context menu. 52 tests pass. States named as in SmartGit. Confirmed (2026-10-21) |
| 8 | Side-by-side diff | ✅ | Monaco (your choice, over CodeMirror / a hand-made view), lazy-loaded; `git/diff.rs` (HEAD / Index / Working Tree, binary, >5 MB, paths outside the repo refused); top/bottom layout with a splitter; double-click → separate diff window (doesn't block the main window). 61 tests pass |
| 9 | Commit + amend | ✅ | Message through stdin (`-F -`), `--cleanup=whitespace`; checks first (nothing staged / conflicts / no commit yet); missing identity → `IdentityUnknown`; asks before amending a pushed commit. 71 tests pass |
| 10 | Current branch + ahead/behind + fetch | ✅ | Header from `status --branch` (no extra git command), `fetch --all --progress` with progress/cancel, no `--prune`; fetch doesn't block stage/commit; cancel sends SIGTERM + SIGCONT. 77 tests pass |
| 11 | Pull, push | ✅ | Pull = fetch (progress/cancel) + `merge --ff-only` / asks Rebase or Merge when diverged (preselected from `pull.rebase`); push never forces, a new branch asks first then uses `--set-upstream`; `Conflict` / `PushRejected`. 90 tests pass |
| 12 | MoonGit as git's credential helper: `-c credential.helper=` reset, MoonGit first, the user's helpers after it, on every network command; `moongit --git-credential get` answers git without opening a window | ✅ | Your choice (2026-10-28): own credential helper (not GIT_ASKPASS); use a saved account of the same server; ask with a dialog when nothing works. 213 Rust tests. No visible change until step 13 |
| 13a | Saved accounts: `accounts.json` (id, server, user name) + token in the OS keyring (crate `keyring` 4.2); sign-in saved before the retry and undone if the server refuses it; the helper reads them (`MOONGIT_DATA_DIR`); auth errors carry the server | ✅ | 220 Rust tests (keyring faked in tests). Nothing visible on its own |
| 13b | "Sign in to <server>" dialog on an auth failure, then retry (fetch, pull, push, force push, clone, delete on the server); refused logins are undone; when MoonGit has an account for the server, the user's own helpers are left out (so git never sends them "erase") | ✅ | 222 Rust + 38 Vitest. Committed without a test against a real HTTPS server (none available yet) |
| 13c | MoonGit → Accounts…: list of saved accounts (server, user name; never the token), Remove… (file + keychain, confirmed) | ✅ | 223 Rust tests. Choosing an account per repository when a server has several: later, if needed |
| 14 | `user.name` / `user.email` per repository (Repository → Commit Identity…, context menu, Commit dialog; `.git/config`) and per group (group context menu): Name / Email written into the .git/config of the repositories of the group and subgroups: "Only repositories without their own setting" or "Write for all repositories" | ✅ | 229 Rust tests. Never changes ~/.gitconfig |
| 15a | Repository → Login (SSH & HTTPS)… (shows which remotes use each): SSH = this computer's settings or one key (`core.sshCommand = ssh -i '<key>' -o IdentitiesOnly=yes`); HTTPS = MoonGit's accounts or this computer's credential helpers (`moongit.credentials = system`) | ✅ | Your choice (2026-11-01): per repository + per group, in .git/config (terminal git does the same), never ~/.ssh/config. 238 Rust tests |
| 15b | Group → Login (SSH & HTTPS)…: written into the repositories of the group and subgroups, SSH and HTTPS each on their own; "only repositories without their own setting" or "write for all" | ✅ | 239 Rust tests |

## Phase 2 — history and branches

| # | Feature | Status |
|---|---|---|
| 16a | Commit history: `git/log.rs` (parser + tests), History tab, hand-written virtualized list + paged loading, branch/tag labels | ✅ 98 tests pass |
| 16b | Graph: lane layout algorithm in TS, separate module with tests (Vitest added), graph column drawing | ✅ 11 Vitest + 98 Rust tests |
| 16c | Choose the branches/remotes/tags shown in the graph | ✅ First time = current branch, then remembered per repository. 104 Rust + 15 Vitest |
| 17 | Filter by hash / message, highlight (default) and hide modes | ✅ 111 Rust + 15 Vitest |
| 18 | Click a commit: details, file list, diff | ✅ 117 Rust + 15 Vitest |
| 19a | Branches: checkout (local / remote → creates a tracking branch), create a new branch (from HEAD or from a commit in History) | ✅ 123 Rust + 15 Vitest |
| 19b | Branches: rename, delete local / remote (with confirmation), set / unset upstream | ✅ 129 Rust + 17 Vitest |

## Phase 3 — advanced operations (split into smaller steps when reached)

| # | Feature | Status |
|---|---|---|
| 20a | Stash: create (message, optionally including untracked), Stashes tab (list + content) | ✅ 136 Rust + 17 Vitest |
| 20b | Stash: apply, pop, drop (confirmed), conflicts on apply | ✅ 141 Rust + 17 Vitest |
| 21a | Merge like SmartGit: Fast-Forward / Create Merge-Commit (`--no-ff`) / Merge to Working Tree (`--no-ff --no-commit`) | ✅ 149 Rust + 17 Vitest |
| 21b | Merge in progress: banner + Abort (confirmed), prefilled message, choose merge commit / simple commit when committing | ✅ 155 Rust + 17 Vitest |
| 22a | Rebase (non-interactive) onto a branch/tag/commit, confirmed first (warns about pushed commits) | ✅ 161 Rust + 17 Vitest |
| 22b | Rebase in progress: banner, Continue / Skip / Abort | ✅ 166 Rust + 17 Vitest |
| 22c | Reset the branch to a commit (History → right-click): Soft / Mixed / Hard; Hard is confirmed twice, warns when dropping pushed commits. *Added to the roadmap at your request (2026-10-23), not in CLAUDE.md* | ✅ 173 Rust + 17 Vitest |
| 22d | Discard file changes (Changes tab): back to HEAD, new files go to the Trash (crate `trash`). *Added at your request (2026-10-24), not in CLAUDE.md* | ✅ 179 Rust + 17 Vitest |
| 23 | Cherry-pick: Commit or Apply to Working Tree (`--no-commit`); waiting after a conflict: Continue / Abort | ✅ 186 Rust + 17 Vitest |
| 24a | Conflicts: Take Ours / Take Theirs (with the real name of each side), Mark Resolved (checks for conflict markers). State detection + Continue/Abort already came with 21b/22b/23 | ✅ 191 Rust + 17 Vitest; Take Ours/Theirs is only in the context menu (since 24b) |
| 24b | 3-column Conflict Solver window like SmartGit (double-click a conflicted file): ours / editable result / theirs, Take Left/Right/Both per block, Save & Mark Resolved. *Replaces the "external compare tool", at your request* | ✅ 193 Rust + 28 Vitest |

## Added on request (after Phase 3)

| # | Feature | Status |
|---|---|---|
| 25 | Branch list in the sidebar like SmartGit: Local (✓, ↑↓, gone) / Remotes grouped by remote; double-click checks out, right-click: checkout / merge / rebase / new / rename / delete | ✅ 196 Rust + 28 Vitest |
| 26 | Add a remote (Add Remote…): + button / right-click "Remotes" in the Branches panel; name + URL, checked before `git remote add --`. Muted URL next to the remote name (tokens masked), right-click → Edit URL | ✅ 203 Rust + 30 Vitest |
| 27 | Force push: ▾ next to Push → Force Push…; preview of the commits the server loses, confirmation, `--force-with-lease=<ref>:<id seen at the last fetch>`; refused (nothing lost) if someone pushed since | ✅ 208 Rust + 30 Vitest |
| 28 | Removed the branch line under the repo name (the sidebar's Branches panel shows it); "Upstream of …" moved to the panel's right-click menu; a warning line remains only for a detached HEAD. Toolbar: Fetch / Pull / Push / Refresh moved to the left, then a separator, then the file actions. Colored 24px icons above the labels on the toolbar buttons (`AppIcon.vue`, hand-drawn duotone SVG, no icon library; shared `button.tool` style). Stash Apply / Apply and Drop (was "Pop") / Drop moved from the Stashes tab to the toolbar, with icons; the Stash… button only on the Changes tab. Sidebar: orange repository icon, bigger two-tone yellow folder icons | ✅ 30 Vitest, frontend only |
| 29 | Commit… button on the toolbar (Changes tab, after Stash…) opening a Commit dialog: message, Amend (or Merge/Simple commit during a merge), Commit / Commit and Push (pushes through the toolbar's Push); the commit area at the bottom is removed | ✅ 30 Vitest, frontend only |
| 30 | History time column: "Today, 14:32" / "Yesterday, 09:10" by local calendar day, full date otherwise (`commitTime.ts`) | ✅ 35 Vitest, frontend only |
| 31a | Menu bar: MoonGit / Repository (was File) / Edit (+ Copy Repository Path) / View (tabs, Refresh ⌘R, Theme placeholder) / Remote / Local / Branch / Window; items run the same functions as the buttons through a command store and are greyed out when not possible | ✅ 208 Rust + 35 Vitest |
| 31b | Branch → Checkout… (⇧⌘C): dialog with a search box (every typed word must match), local + remote branches, ↑/↓/Enter | ✅ 38 Vitest, frontend only |
| 32 | Welcome screen (no repository selected): your moon base picture (`src/assets/background.svg`), large and centered, with CSS animations inside the SVG (twinkling stars, shooting star, signal waves, light flowing along the paths; still when Reduce motion is on), plus a one-line hint | ✅ frontend only |
| 33 | Dark mode: View → Theme ▸ System / Light / Dark, remembered; Tauri's app-level `setTheme` switches every window, the web views follow (`prefers-color-scheme`), so CSS and Monaco need no change | ✅ frontend + one permission |
| 34 | Drag a repository onto a group (into it), onto another repository (its group) or onto empty space (top level); a closed group opens when hovered; HTML5 drag and drop, no library; needs `dragDropEnabled: false` on the main window (Tauri's file drop swallowed the `drop` event on macOS too) | ✅ frontend + one window option |
| 35a | Drag a whole group into another group or out to the top level; not into itself or its own subgroups, not next to a group of the same name | ✅ | 241 Rust tests |
| 35b | Select several repositories (⌘-click, ⇧-click): drag them together; context menu Move To / Commit Identity… / Login… / Remove from List (35c merged in) | ✅ | 241 Rust + 42 Vitest |

---

## Decisions made
- App name **MoonGit**, identifier `com.github.halab4dev.moongit`.
- Package manager: **pnpm**.
- Removed the template's `opener` plugin (not needed yet).
- Pinia v4, "setup store" style (`ref`/`computed`/functions).
- The repository list is stored in `<app data>/repos.json` (`{ version, repos }`); a broken file is renamed to `repos.json.broken`.
- Repository ids are generated from the time (hex), without the `uuid` crate.
- Removing a repository from the list never touches its folder on disk.
- Groups are stored flat (`parentId`); deleting a group moves its content up one level. Group names under the same parent are unique (case-insensitive).
- Sidebar actions use Tauri's native menus; the × button of step 3 was replaced by "Remove from List".
- Long operations: worker thread (`spawn_blocking`) + reading stderr, instead of `tokio::process` (easier to read, no extra library). Each operation has an `opId`; progress goes through the `git-progress` event.
- Cancel = SIGTERM to the whole process group (crate `rustix`, unix only). On Windows, only `git` itself is stopped for now.
- A failed/cancelled clone deletes the target folder only if it didn't exist before the clone.
- Commands working on a repository take a `repo_id` (Rust looks up the path) and run in the background through `run_blocking`.
- Status refreshes when the window gets the focus + a Refresh button (no file watcher yet).
- Stage/unstage always uses `--literal-pathspecs`; conflicted files couldn't be staged until the conflict step.
- `vite.config.ts` transpiles JS down to `safari15` / `chrome105` (the macOS 12 webview = system WebKit ≈ Safari 15.6, even if the Safari app is newer). Checked with the system's `jsc`.
- Fetch = `git fetch --all --progress`, without `--prune` (pruning branches deleted on the server will be a separate option later).
- Pull: asks Rebase/Merge every time the branch has diverged (your choice), preselected from git config; when only behind, fast-forwards without asking. Done with fetch + a local merge/rebase against `@{upstream}`, not `git pull`.
- The Push button never forces. Force push (step 27) is a separate menu action, always confirmed with the list of commits the server loses, and always `--force-with-lease=<remote ref>:<id from the last fetch>` (never plain `--force`, never the bare `--force-with-lease`).
- History: `git log --date-order` from `--branches --remotes --tags HEAD` (not `--all`, to keep stashes out), pages of 500 commits via `--skip`; hand-written virtualized list (no extra library).
- Diffs use Monaco, loading only the core + syntax highlighting + workers, on the first diff opened. Three modes: Working Tree (Index→WT), Staged (HEAD→Index), Conflict (HEAD→WT).
- The State column uses SmartGit's file state names (Staged, Added, Modified, Missing…); sorted by importance, Conflicted first.
- Dropped the `greet` example in step 2, replaced by a `check_repo` test form (itself replaced by "Add repository" in step 3).
- `AppError` is a struct `{ kind, message }`; git runs with `GIT_TERMINAL_PROMPT=0` and `LC_ALL=C`.
- Tests use the `tempfile` crate (dev-dependency only).
- TypeScript tests use **Vitest** (devDependency), run with `pnpm test`.
- History ref selection: first time = current branch + its upstream (your request), then saved per repository in localStorage (`null` = all, including new branches); Rust only accepts existing refs + `--end-of-options`.
- History filter: one box for both an id prefix (when all hex) and the full message, case-insensitive; searched in Rust over the whole history; Hide mode rewrites parents to the nearest matching ancestor.
- Commit details: merges are compared with their first parent (like SmartGit); only hex commit ids are accepted.
- Branches: `git switch` (not `checkout`), `--no-guess`, never `--force`; checking out a remote branch = creating a local branch of the same name that tracks it. Deleting an unmerged branch: asks a second time, then `--force`. Deleting on the server uses `refs/heads/<name>` so a tag is never hit.
- Stash: checked before stashing (git reports success even when there is nothing to stash); stashes are identified by id, not by index; content shown with `CommitDetails` (W against HEAD at stash time, U for untracked files). Apply doesn't use `--index` (every change comes back unstaged); a pop with conflicts keeps the stash.
- Merge works like SmartGit (your choice, 2026-10-23): three options Fast-Forward / Create Merge-Commit / Merge to Working Tree; the default is always Create Merge-Commit (your choice). Merge state is read from `MERGE_HEAD`/`MERGE_MSG` (via `--git-path`) in `read_status`; Simple commit = delete `MERGE_HEAD`, then commit. The app writes the merge message itself (full ref names are passed to git).
- Rebase: always asks first (counts the commits to replay and the pushed ones); refused with uncommitted changes, `--no-autostash`; no merge/rebase/commit while a rebase is in progress. Continue runs with `GIT_EDITOR=true` (keeps the original message); Skip / Abort always ask first. The status banner is the shared `OperationBanner`.
- Reset added to the roadmap (step 22c, your request 2026-10-23): the three modes of SmartGit; Hard always asks a second time. Mixed by default; the full commit id is passed to `git reset` (git 2.33 doesn't accept `--end-of-options` for this command).
- Discard (step 22d, your request 2026-10-24): like SmartGit, back to HEAD for both staged and unstaged changes; files not in HEAD go to the Trash (your choice, crate `trash`) instead of being deleted. Only paths present in the status are handled; conflicted files can't be discarded.
- Cherry-pick: one commit from History; a merge commit is taken relative to its first parent (`--mainline 1`); an empty cherry-pick (already applied) is aborted automatically and reported; the waiting state is read from `CHERRY_PICK_HEAD`.
- Conflicts: buttons show the real names of ours/theirs (a rebase swaps them); Take side = `checkout --ours/--theirs` + `add`, the side that deleted the file = `rm`; Mark Resolved checks for all 3 kinds of markers, asks again if any are left.
- Conflict Solver (your request 2026-10-24): a separate 3-column window like SmartGit, replacing the Take Ours/Theirs toolbar buttons and the external tool; hand-written Myers diff (`src/conflict/lineDiff.ts`) to highlight changes and scroll the side columns; non-UTF-8 files are treated as binary (not editable).
- Branch panel (step 25): the sidebar is split into a top (repositories) / bottom (branches) part with a splitter; the `repoActivity` store connects the sidebar with RepoView (RepoView stays the only owner of the status).
- Remotes (step 26): URLs are read with `git config --get-regexp` (the raw value, before `insteadOf`); a password/token inside a URL is masked in the list; names and URLs starting with `-` are refused.
- Menu bar (step 31a): built in JavaScript (`setAsAppMenu`), no Rust menu code. Items only send a command id to the `menuCommands` store; components register what they can do while mounted, so the menu follows the same conditions as the buttons. Copy Repository Path uses the official clipboard plugin (write-text permission only).
- Theme (step 33): Tauri's app-level `setTheme` (null = follow macOS) instead of a `data-theme` attribute; the choice is kept in the main window's localStorage and applied at start.
- Credentials (step 12): MoonGit is its own git credential helper, passed with `-c` per command (no git config file is changed). The user's configured helpers are re-added after it, so nothing changes when MoonGit has no saved login. Secrets only travel through the helper's stdout pipe.
- Accounts (step 13a): `accounts.json` holds no secret; tokens are in the OS keyring under service `com.github.halab4dev.moongit`, user = account id. Same server + user name = same account; the latest sign-in is used first for its server. A sign-in is saved before git tries it, then kept or undone.
- Group identities (step 14, your choice): written into each repository's .git/config (terminal git uses them too); two choices for the whole group: only repositories without their own setting, or all of them. The group's last identity is kept in repos.json to prefill the dialog.
- SSH key & HTTPS login (step 15a): stored in the repository's .git/config (`core.sshCommand`, `moongit.credentials`); "this computer's" = not set. With `moongit.credentials = system`, MoonGit adds no credential helper and never shows its Sign in dialog for that repository.
- Graph: lanes never shift sideways, free columns are reused; `LaneLayout` is computed incrementally per page; one SVG per row, at most 20 lanes shown.

## Open items
- [x] `sudo xcode-select --switch /Library/Developer/CommandLineTools` (checked 2026-10-26: points to the Command Line Tools).
- [x] First commit `dd92830` (steps 1–2).
- [x] Steps 3–11 confirmed and committed.
- [x] Steps 16a–19b confirmed and committed.
- [x] Steps 20a–24b confirmed and committed (22c was committed on 2026-10-24, when moving on to 22d). Phase 3 done.
- [x] Steps 25–35b and 12–15 confirmed and committed. CLAUDE.md roadmap complete (2026-11-01). README and MIT license added.
- [ ] Try steps 13b / 13c against a real HTTPS server when one is available (tables in `learning/13b-…`, `13c-…`).
