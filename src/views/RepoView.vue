<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { Menu } from "@tauri-apps/api/menu";
import CommitBox from "../components/CommitBox.vue";
import DiffPanel from "../components/DiffPanel.vue";
import FileTable from "../components/FileTable.vue";
import HistoryView from "./HistoryView.vue";
import OperationBanner from "../components/OperationBanner.vue";
import StashView from "./StashView.vue";
import SyncButtons from "../components/SyncButtons.vue";
import { buildRows, type FileRow } from "../components/fileRows";
import { openConflictWindow, openDiffWindow } from "../windows";
import { useSplitRatio } from "../composables/useSplitRatio";
import { getStatus } from "../api/status";
import { discardChanges, resolveConflicts, stageFiles, unstageFiles } from "../api/stage";
import { commitChanges } from "../api/commit";
import { createStash } from "../api/stash";
import { abortMerge } from "../api/merge";
import { runRebaseAction } from "../api/rebase";
import { runCherryPickAction } from "../api/cherryPick";
import type { CherryPickAction } from "../types/cherryPick";
import type { RebaseAction } from "../types/rebase";
import { ask } from "@tauri-apps/plugin-dialog";
import { useDialogStore } from "../stores/dialogs";
import { useRepoActivity } from "../stores/repoActivity";
import { toAppError } from "../api/errors";
import type { AppError } from "../types/error";
import type { RepoEntry } from "../types/repo";
import type { ConflictChoice, RepoStatus } from "../types/status";

const props = defineProps<{
  repo: RepoEntry;
}>();

// Kept in this view, not in a store: App.vue rebuilds RepoView (via :key)
// for every repo, so each repo starts with fresh state.
const status = ref<RepoStatus | null>(null);
// +1 every time a new status arrives; tells the diff panel to reload too.
const statusVersion = ref(0);
const busy = ref(false);
// Set by SyncButtons while a pull may change files.
const syncLocked = ref(false);
// Set by StashView while a stash is applied, popped or dropped.
const stashLocked = ref(false);
// Staging and committing must wait for all of them.
const working = computed(() => busy.value || syncLocked.value || stashLocked.value);

// The branch panel in the sidebar (step 25) follows this view: it reloads
// when the status changes, waits while we work, and hands us the status of
// its own actions (checkout…).
const activity = useRepoActivity();
watch(statusVersion, () => activity.statusChanged());
watch(working, (value) => (activity.busy = value), { immediate: true });
watch(
  () => activity.incomingStatus,
  (incoming) => {
    if (incoming !== null) {
      activity.incomingStatus = null;
      applyStatus(incoming);
    }
  },
);
const error = ref<AppError | null>(null);
const selectedPaths = ref<string[]>([]);

// Changes (working tree), History or Stashes. Remembered for all repos.
type Tab = "changes" | "history" | "stashes";
const TABS: Tab[] = ["changes", "history", "stashes"];
const TAB_KEY = "moongit.repoView.tab";
const tab = ref<Tab>(readTab());

function selectTab(next: Tab) {
  tab.value = next;
  try {
    localStorage.setItem(TAB_KEY, next);
  } catch {
    // Not remembered; fine.
  }
}

function readTab(): Tab {
  try {
    const saved = localStorage.getItem(TAB_KEY);
    return TABS.find((t) => t === saved) ?? "changes";
  } catch {
    return "changes";
  }
}
const commitMessage = ref("");
const amend = ref(false);
// During a merge: commit it as an ordinary commit instead of the merge commit.
const simpleCommit = ref(false);

const merge = computed(() => status.value?.merge ?? null);
const rebase = computed(() => status.value?.rebase ?? null);
const conflictCount = computed(() => status.value?.conflicted.length ?? 0);
const cherryPick = computed(() => status.value?.cherryPick ?? null);

// A conflict error says the same as the coloured bar (Merging / Rebasing /
// Cherry-picking), so don't repeat it below the bar. Without a bar (stash
// apply, cherry-pick to the working tree) the error line is the only notice.
const shownError = computed(() => {
  const hasBar = merge.value !== null || rebase.value !== null || cherryPick.value !== null;
  return error.value?.kind === "conflict" && hasBar ? null : error.value;
});
// e.g. `"main" onto 1234abcd, commit 1 of 2 (5678ef90 Fix login)`
const rebaseTitle = computed(() => {
  const state = rebase.value;
  if (state === null) {
    return "";
  }
  const where = `"${state.branch}" onto ${state.onto.slice(0, 8)}, commit ${state.step} of ${state.total}`;
  return state.stoppedAt ? `${where} (${state.stoppedAt})` : where;
});
// First line of git's prepared message, e.g. "Merge branch 'feature' into main".
const mergeTitle = computed(() => merge.value?.message.split("\n")[0] || "Merge");

// A merge started (or the app opened on one): fill in git's prepared message,
// unless something is typed already. When it ends without our commit
// (aborted, or committed in a terminal), take that message away again.
watch(
  () => merge.value?.message ?? null,
  (next, previous) => {
    if (next !== null) {
      // The Amend box is hidden during a merge (git can't amend then).
      amend.value = false;
      if (commitMessage.value.trim() === "") {
        commitMessage.value = next;
      }
    } else if (next === null && previous !== null && previous !== undefined) {
      if (commitMessage.value === previous) {
        commitMessage.value = "";
      }
      simpleCommit.value = false;
    }
  },
);

const rows = computed(() => (status.value === null ? [] : buildRows(status.value)));
const selectedRows = computed(() => {
  const selected = new Set(selectedPaths.value);
  return rows.value.filter((row) => selected.has(row.path));
});
const stagedCount = computed(() => status.value?.staged.length ?? 0);
const diffRow = computed(() => (selectedRows.value.length === 1 ? selectedRows.value[0] : null));

// Conflicted files are left out on purpose: `git add` on them means
// "conflict resolved", which goes through Mark Resolved (step 24a).
const pathsToStage = computed(() =>
  selectedRows.value.filter((row) => row.hasUnstaged && row.state !== "conflicted").map((row) => row.path),
);

// A rename is undone by unstaging both its new and its old path.
const pathsToUnstage = computed(() =>
  selectedRows.value
    .filter((row) => row.hasStaged)
    .flatMap((row) => (row.origPath ? [row.path, row.origPath] : [row.path])),
);

// Every git action here returns the new status. One at a time: while one
// runs, the others (and focus refreshes) are ignored.
// Resolves with true when the action succeeded.
async function run(action: () => Promise<RepoStatus>): Promise<boolean> {
  if (working.value) {
    return false;
  }
  busy.value = true;
  try {
    status.value = await action();
    statusVersion.value++;
    error.value = null;
    return true;
  } catch (err) {
    error.value = toAppError(err);
    return false;
  } finally {
    busy.value = false;
  }
}

function refresh() {
  return run(() => getStatus(props.repo.id));
}

// ---- conflicts (step 24a) ----

const conflictedPaths = computed(() =>
  selectedRows.value.filter((row) => row.state === "conflicted").map((row) => row.path),
);

// What "ours" and "theirs" are right now, in words. Careful with rebase:
// there "ours" is the branch rebased onto, and "theirs" your own commit.
const sides = computed(() => {
  const s = status.value;
  if (s?.rebase) {
    return {
      ours: `${s.rebase.onto.slice(0, 8)} (rebasing onto)`,
      theirs: s.rebase.stoppedAt ? `your commit ${s.rebase.stoppedAt.slice(0, 8)}` : "your commit",
    };
  }
  const branch = s?.branch.name ?? "HEAD";
  if (s?.cherryPick) {
    return { ours: branch, theirs: s.cherryPick.commit.slice(0, 8) };
  }
  if (s?.merge) {
    // "Merge branch 'feature' into main" → feature
    const merged = /'([^']+)'/.exec(s.merge.message)?.[1];
    return { ours: branch, theirs: merged ?? "the merged branch" };
  }
  // A stash apply or a cherry-pick to the working tree: no state to name it.
  return { ours: branch, theirs: "the other side" };
});

// Buttons stay short: long names are cut.
function short(name: string): string {
  return name.length > 24 ? `${name.slice(0, 23)}…` : name;
}

async function resolveSelected(choice: ConflictChoice) {
  const paths = conflictedPaths.value;
  if (working.value || paths.length === 0) {
    return;
  }
  const list = paths.length === 1 ? `"${paths[0]}"` : `${paths.length} files`;
  // Taking a side replaces the whole file: edits made to it are lost.
  if (choice !== "resolved") {
    const side = choice === "ours" ? sides.value.ours : sides.value.theirs;
    const confirmed = await ask(
      `Replace ${list} with the version from ${side}?\n\nThe changes from the other side, and any edits you made to the file, are lost.`,
      { title: choice === "ours" ? "Take Ours" : "Take Theirs", kind: "warning", okLabel: "Replace", cancelLabel: "Cancel" },
    );
    if (!confirmed) {
      return;
    }
  }
  const resolve = (force: boolean) => () => resolveConflicts(props.repo.id, paths, choice, force);
  if (await run(resolve(false))) {
    return;
  }
  // Markers left: maybe on purpose (e.g. a file that documents them); ask.
  if (error.value?.kind === "conflictMarkers") {
    const confirmed = await ask(`${error.value.message}\n\nMark as resolved anyway?`, {
      title: "Conflict Markers Left",
      kind: "warning",
      okLabel: "Mark Resolved",
      cancelLabel: "Keep Editing",
    });
    if (confirmed) {
      await run(resolve(true));
    }
  }
}

// Conflicted files are left out, as for staging (resolve them instead).
const rowsToDiscard = computed(() => selectedRows.value.filter((row) => row.state !== "conflicted"));

// States whose file HEAD doesn't have: discarding moves it to the Trash.
const NEW_FILE_STATES = new Set(["untracked", "added", "addedModified", "intentToAdd", "renamed", "renamedModified", "copied"]);

async function discardSelected() {
  const rows = rowsToDiscard.value;
  if (working.value || rows.length === 0) {
    return;
  }
  // The file list in the question, shortened for long selections.
  const SHOWN = 10;
  const names = rows.slice(0, SHOWN).map((row) => `• ${row.path}`);
  if (rows.length > SHOWN) {
    names.push(`… and ${rows.length - SHOWN} more`);
  }
  const toTrash = rows.filter((row) => NEW_FILE_STATES.has(row.state)).length;
  const lost = rows.length - toTrash;
  const notes = [];
  if (lost > 0) {
    notes.push(`The changes to ${lost} tracked file(s) are lost for good.`);
  }
  if (toTrash > 0) {
    notes.push(`${toTrash} new or untracked file(s) are moved to the Trash.`);
  }
  const confirmed = await ask(`Discard the changes of ${rows.length} file(s)?\n\n${names.join("\n")}\n\n${notes.join(" ")}`, {
    title: "Discard Changes",
    kind: "warning",
    okLabel: "Discard",
    cancelLabel: "Cancel",
  });
  if (confirmed) {
    await run(() => discardChanges(props.repo.id, rows.map((row) => row.path)));
  }
}

function stageSelected() {
  const paths = pathsToStage.value;
  return paths.length === 0 ? Promise.resolve() : run(() => stageFiles(props.repo.id, paths));
}

function unstageSelected() {
  const paths = pathsToUnstage.value;
  return paths.length === 0 ? Promise.resolve() : run(() => unstageFiles(props.repo.id, paths));
}

async function commitStaged() {
  const asSimple = merge.value !== null && simpleCommit.value;
  if (await run(() => commitChanges(props.repo.id, commitMessage.value, amend.value, asSimple))) {
    commitMessage.value = "";
    amend.value = false;
    simpleCommit.value = false;
  }
}

// After fetch/pull/push: reload first, then show the error (a successful
// reload clears the error line, so the order matters).
async function onSyncFinished(syncError: AppError | null) {
  await refresh();
  if (syncError !== null) {
    error.value = syncError;
  }
}

// A branch was checked out or created (from the header menu or the History
// tab): show the new status, and let the history reload.
function applyStatus(next: RepoStatus) {
  status.value = next;
  statusVersion.value++;
  error.value = null;
}

// A stash action failed: it may still have changed files (conflicts), so
// reload, then show the error (same order as onSyncFinished).
async function onStashFailed(stashError: AppError) {
  await refresh();
  error.value = stashError;
}

// The merge's own changes leave the working tree; changes made before it are
// kept (git refuses to merge over files with local changes it would touch).
async function confirmAbortMerge() {
  if (working.value) {
    return;
  }
  const confirmed = await ask(
    `Abort "${mergeTitle.value}"?\n\nThe changes the merge brought into the working tree are removed, including any conflict fixes you made. The branch stays where it was.`,
    { title: "Abort Merge", kind: "warning", okLabel: "Abort Merge", cancelLabel: "Cancel" },
  );
  if (confirmed) {
    await run(() => abortMerge(props.repo.id));
  }
}

// Continue right away; skip and abort throw work away, so they ask first.
async function runRebase(action: RebaseAction) {
  const state = rebase.value;
  if (working.value || state === null) {
    return;
  }
  if (action === "skip") {
    const confirmed = await ask(
      `Skip ${state.stoppedAt ?? "this commit"}?\n\nIts changes are left out of "${state.branch}", including any conflict fixes you made. The remaining commits are replayed.`,
      { title: "Skip Commit", kind: "warning", okLabel: "Skip Commit", cancelLabel: "Cancel" },
    );
    if (!confirmed) {
      return;
    }
  }
  if (action === "abort") {
    const confirmed = await ask(
      `Abort the rebase of "${state.branch}"?\n\nThe branch goes back to where it was before the rebase. Conflict fixes you made are lost.`,
      { title: "Abort Rebase", kind: "warning", okLabel: "Abort Rebase", cancelLabel: "Cancel" },
    );
    if (!confirmed) {
      return;
    }
  }
  if (!(await run(() => runRebaseAction(props.repo.id, action)))) {
    // The next commit may have conflicted: the files changed, so reload
    // (keeping the explanation, like after a failed sync).
    const failure = error.value;
    await refresh();
    error.value = failure;
  }
}

// Continue right away; abort asks first.
async function runCherryPick(action: CherryPickAction) {
  const state = cherryPick.value;
  if (working.value || state === null) {
    return;
  }
  if (action === "abort") {
    const confirmed = await ask(
      `Abort the cherry-pick of ${state.commit.slice(0, 8)} "${state.subject}"?\n\nIts changes leave the working tree, including any conflict fixes you made.`,
      { title: "Abort Cherry-Pick", kind: "warning", okLabel: "Abort Cherry-Pick", cancelLabel: "Cancel" },
    );
    if (!confirmed) {
      return;
    }
  }
  if (!(await run(() => runCherryPickAction(props.repo.id, action)))) {
    const failure = error.value;
    await refresh();
    error.value = failure;
  }
}

// "Stash…": message (optional) and whether to include untracked files.
// Errors are shown inside the dialog, which stays open.
function askStash() {
  if (working.value) {
    return;
  }
  useDialogStore().askName({
    title: "Stash Changes",
    confirmLabel: "Stash",
    initial: "",
    placeholder: "Message (optional)",
    allowEmpty: true,
    option: { label: "Include untracked files", initial: false },
    submit: async (message, includeUntracked) => {
      if (working.value) {
        return "Another operation is running. Try again when it has finished.";
      }
      busy.value = true;
      try {
        applyStatus(await createStash(props.repo.id, message, includeUntracked));
        return null;
      } catch (err) {
        return toAppError(err).message;
      } finally {
        busy.value = false;
      }
    },
  });
}

async function showRowMenu() {
  const menu = await Menu.new({
    items: [
      { text: "Stage", enabled: pathsToStage.value.length > 0, action: () => void stageSelected() },
      { text: "Unstage", enabled: pathsToUnstage.value.length > 0, action: () => void unstageSelected() },
      { item: "Separator" },
      { text: "Discard…", enabled: rowsToDiscard.value.length > 0, action: () => void discardSelected() },
      ...(conflictedPaths.value.length > 0
        ? [
            { item: "Separator" as const },
            { text: "Resolve…", enabled: conflictRow.value !== null, action: () => void openSolver() },
            // Whole-file choices stay here for files the solver can't show
            // (binary, or deleted on one side).
            { text: `Take Ours (${short(sides.value.ours)})…`, action: () => void resolveSelected("ours") },
            { text: `Take Theirs (${short(sides.value.theirs)})…`, action: () => void resolveSelected("theirs") },
            { text: "Mark Resolved", action: () => void resolveSelected("resolved") },
          ]
        : []),
    ],
  });
  await menu.popup();
}

// Double-click: the whole file with its changes in a separate, resizable
// window; for a conflicted file, the conflict solver (three columns).
async function openInWindow(row: FileRow) {
  try {
    if (row.state === "conflicted") {
      await openConflictWindow(props.repo.id, props.repo.name, row, sides.value.ours, sides.value.theirs);
    } else {
      await openDiffWindow(props.repo.id, props.repo.name, row);
    }
  } catch (err) {
    error.value = toAppError(err);
  }
}

// "Resolve…" works on one conflicted file at a time.
const conflictRow = computed(() => {
  const rows = selectedRows.value.filter((row) => row.state === "conflicted");
  return rows.length === 1 ? rows[0] : null;
});

function openSolver() {
  if (conflictRow.value !== null) {
    void openInWindow(conflictRow.value);
  }
}

// Splitter between the file table (top) and the diff (bottom).
const { ratio: filesRatio, startResize } = useSplitRatio("split", "moongit.repoView.filesRatio", 0.35);

// Coming back to the window (e.g. after editing in an IDE) usually means files changed.
onMounted(() => {
  refresh();
  window.addEventListener("focus", refresh);
});

onUnmounted(() => {
  window.removeEventListener("focus", refresh);
});
</script>

<template>
  <section class="repo-view">
    <header class="repo-header">
      <h1>{{ repo.name }}</h1>
      <p class="muted path">{{ repo.path }}</p>
      <!-- The current branch is shown in the sidebar's branch panel; only a
           detached HEAD is called out here (it has no row there). -->
      <p v-if="status && status.branch.name === null && status.branch.commit !== null" class="detached">
        HEAD detached at {{ status.branch.commit.slice(0, 7) }}: new commits here belong to no branch. Check out a
        branch, or create one from History.
      </p>
    </header>

    <nav class="tabs">
      <button :class="{ active: tab === 'changes' }" @click="selectTab('changes')">
        Changes<span v-if="rows.length > 0" class="count">{{ rows.length }}</span>
      </button>
      <button :class="{ active: tab === 'history' }" @click="selectTab('history')">History</button>
      <button :class="{ active: tab === 'stashes' }" @click="selectTab('stashes')">Stashes</button>
    </nav>

    <div class="toolbar">
      <template v-if="tab === 'changes'">
        <button :disabled="working || pathsToStage.length === 0" title="Stage selected files" @click="stageSelected">
          Stage
        </button>
        <button :disabled="working || pathsToUnstage.length === 0" title="Unstage selected files" @click="unstageSelected">
          Unstage
        </button>
        <button
          :disabled="working || rowsToDiscard.length === 0"
          title="Throw away the changes of the selected files (new files go to the Trash)"
          @click="discardSelected"
        >
          Discard…
        </button>
        <!-- Only when conflicted files are selected. -->
        <template v-if="conflictedPaths.length > 0">
          <span class="separator"></span>
          <button
            class="primary"
            :disabled="conflictRow === null"
            title="Open the conflict solver for this file (or double-click it)"
            @click="openSolver"
          >
            Resolve…
          </button>
          <button
            :disabled="working"
            title="You fixed the file by hand: stage it as it is now"
            @click="resolveSelected('resolved')"
          >
            Mark Resolved
          </button>
        </template>
      </template>
      <button
        v-if="tab !== 'history'"
        :disabled="working || rows.length === 0"
        title="Put the local changes aside and clean the working tree"
        @click="askStash"
      >
        Stash…
      </button>
      <span class="spacer"></span>
      <SyncButtons
        v-if="status"
        v-model:locked="syncLocked"
        :repo-id="repo.id"
        :branch="status.branch"
        :busy="busy"
        @finished="onSyncFinished"
      />
      <button :disabled="working" title="Reload changed files and history" @click="refresh">Refresh</button>
    </div>

    <!-- Blue while the merge just waits for its commit, red while files conflict. -->
    <OperationBanner v-if="merge" label="Merging:" :conflicts="conflictCount > 0">
      {{ mergeTitle }}.
      <template v-if="conflictCount > 0">{{ conflictCount }} file(s) have conflicts: double-click a file to resolve it.</template>
      <template v-else>Review the changes, then commit to finish.</template>
      <template #actions>
        <button :disabled="working" title="Undo the merge (git merge --abort)" @click="confirmAbortMerge">
          Abort Merge
        </button>
      </template>
    </OperationBanner>

    <OperationBanner v-if="rebase" label="Rebasing:" :conflicts="conflictCount > 0">
      {{ rebaseTitle }}.
      <template v-if="conflictCount > 0">{{ conflictCount }} file(s) have conflicts: double-click a file to resolve it, then Continue.</template>
      <template v-else>Stage your fixes, then Continue to replay the remaining commits.</template>
      <template #actions>
        <button
          class="primary"
          :disabled="working || conflictCount > 0"
          title="Commit the fixed commit and replay the rest (git rebase --continue)"
          @click="runRebase('continue')"
        >
          Continue
        </button>
        <button :disabled="working" title="Leave this commit out (git rebase --skip)" @click="runRebase('skip')">
          Skip…
        </button>
        <button :disabled="working" title="Undo the whole rebase (git rebase --abort)" @click="runRebase('abort')">
          Abort…
        </button>
      </template>
    </OperationBanner>

    <OperationBanner v-if="cherryPick" label="Cherry-picking:" :conflicts="conflictCount > 0">
      {{ cherryPick.commit.slice(0, 8) }} "{{ cherryPick.subject }}".
      <template v-if="conflictCount > 0">{{ conflictCount }} file(s) have conflicts: double-click a file to resolve it, then Continue.</template>
      <template v-else>Stage your fixes, then Continue to commit it.</template>
      <template #actions>
        <button
          class="primary"
          :disabled="working || conflictCount > 0"
          title="Commit the result with the original message (git cherry-pick --continue)"
          @click="runCherryPick('continue')"
        >
          Continue
        </button>
        <button :disabled="working" title="Undo the cherry-pick (git cherry-pick --abort)" @click="runCherryPick('abort')">
          Abort…
        </button>
      </template>
    </OperationBanner>

    <p v-if="shownError" class="err message">{{ shownError.message }}</p>

    <!-- v-show, not v-if: switching tabs keeps the selection and the diff. -->
    <div v-show="tab === 'changes'" class="pane">
      <div ref="split" class="split">
        <div class="files" :style="{ flexBasis: `${filesRatio * 100}%` }">
          <template v-if="status">
            <p v-if="rows.length === 0" class="muted">Nothing to commit, working tree clean.</p>
            <FileTable
              v-else
              v-model:selected="selectedPaths"
              :status="status"
              @row-menu="showRowMenu"
              @row-open="openInWindow"
            />
          </template>
          <p v-else-if="busy" class="muted">Loading…</p>
        </div>
        <div class="splitter" title="Drag to resize" @mousedown="startResize"></div>
        <div class="diff">
          <DiffPanel :repo-id="repo.id" :row="diffRow" :selected-count="selectedRows.length" :version="statusVersion" />
        </div>
      </div>

      <CommitBox
        v-model:message="commitMessage"
        v-model:amend="amend"
        v-model:simple="simpleCommit"
        :repo-id="repo.id"
        :merging="merge !== null"
        :staged-count="stagedCount"
        :busy="working || rebase !== null || cherryPick !== null"
        @commit="commitStaged"
      />
    </div>

    <!-- v-if: the history is loaded when the tab is opened, and dropped when left. -->
    <HistoryView
      v-if="tab === 'history'"
      class="pane"
      :repo-id="repo.id"
      :version="statusVersion"
      @status-changed="applyStatus"
    />
    <StashView
      v-if="tab === 'stashes'"
      v-model:locked="stashLocked"
      class="pane"
      :repo-id="repo.id"
      :version="statusVersion"
      :busy="working"
      @status-changed="applyStatus"
      @failed="onStashFailed"
    />
  </section>
</template>

<style scoped>
/* Full-height column: header and toolbar on top, the split area takes the rest. */
.repo-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px 20px 8px;
}

.repo-header {
  margin-bottom: 10px;
}

h1 {
  margin: 0;
  font-size: 20px;
}

.path {
  margin: 2px 0 0;
  word-break: break-all;
}

.detached {
  margin: 4px 0 0;
  color: var(--warning);
}

.toolbar {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
}

.spacer {
  flex: 1;
}

.separator {
  width: 1px;
  align-self: stretch;
  background: var(--border);
}

.muted {
  color: var(--text-muted);
}

.message {
  white-space: pre-wrap;
  word-break: break-word;
}

.tabs {
  display: flex;
  gap: 2px;
  margin-bottom: 8px;
  border-bottom: 1px solid var(--border);
}

.tabs button {
  border: none;
  border-bottom: 2px solid transparent;
  border-radius: 0;
  background: transparent;
  padding: 4px 12px;
  color: var(--text-muted);
}

.tabs button.active {
  border-bottom-color: var(--accent);
  color: var(--text);
  font-weight: 600;
}

.count {
  margin-left: 6px;
  padding: 0 6px;
  border-radius: 8px;
  background: var(--bg-sidebar);
  font-size: 11px;
}

/* Each tab fills the rest of the height. */
.pane {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.split {
  flex: 1;
  min-height: 0; /* let the children shrink instead of overflowing */
  display: flex;
  flex-direction: column;
}

.files {
  flex-grow: 0;
  flex-shrink: 0;
  overflow: auto;
}

.splitter {
  flex: 0 0 6px;
  cursor: row-resize;
}

.splitter:hover {
  background: var(--border);
}

.diff {
  flex: 1;
  min-height: 0;
}
</style>
