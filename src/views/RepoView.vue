<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { Menu } from "@tauri-apps/api/menu";
import BranchStatus from "../components/BranchStatus.vue";
import CommitBox from "../components/CommitBox.vue";
import DiffPanel from "../components/DiffPanel.vue";
import FileTable from "../components/FileTable.vue";
import HistoryView from "./HistoryView.vue";
import StashView from "./StashView.vue";
import SyncButtons from "../components/SyncButtons.vue";
import { buildRows, type FileRow } from "../components/fileRows";
import { openDiffWindow } from "../windows";
import { useSplitRatio } from "../composables/useSplitRatio";
import { showBranchMenu, type BranchCallbacks } from "../components/branchMenus";
import { getStatus } from "../api/status";
import { stageFiles, unstageFiles } from "../api/stage";
import { commitChanges } from "../api/commit";
import { createStash } from "../api/stash";
import { abortMerge } from "../api/merge";
import { ask } from "@tauri-apps/plugin-dialog";
import { useDialogStore } from "../stores/dialogs";
import { toAppError } from "../api/errors";
import type { AppError } from "../types/error";
import type { RepoEntry } from "../types/repo";
import type { RepoStatus } from "../types/status";

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
// "conflict resolved", which belongs to conflict handling (step 15).
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

const branchCallbacks: BranchCallbacks = {
  onStatus: applyStatus,
  onError: (err) => (error.value = err),
};

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

function openBranchMenu() {
  if (!working.value) {
    void showBranchMenu(props.repo.id, branchCallbacks);
  }
}

async function showRowMenu() {
  const menu = await Menu.new({
    items: [
      { text: "Stage", enabled: pathsToStage.value.length > 0, action: () => void stageSelected() },
      { text: "Unstage", enabled: pathsToUnstage.value.length > 0, action: () => void unstageSelected() },
    ],
  });
  await menu.popup();
}

// Double-click: the whole file with its changes in a separate, resizable window.
async function openInWindow(row: FileRow) {
  try {
    await openDiffWindow(props.repo.id, props.repo.name, row);
  } catch (err) {
    error.value = toAppError(err);
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
      <BranchStatus v-if="status" :branch="status.branch" @open-menu="openBranchMenu" />
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
    <div v-if="merge" class="merge-banner" :class="{ conflicts: status && status.conflicted.length > 0 }">
      <span>
        <strong>Merging:</strong> {{ mergeTitle }}.
        <template v-if="status && status.conflicted.length > 0">
          {{ status.conflicted.length }} file(s) have conflicts: fix them, then stage them.
        </template>
        <template v-else>Review the changes, then commit to finish.</template>
      </span>
      <span class="spacer"></span>
      <button :disabled="working" title="Undo the merge (git merge --abort)" @click="confirmAbortMerge">Abort Merge</button>
    </div>

    <p v-if="error" class="err message">{{ error.message }}</p>

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
        :busy="working"
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

.toolbar {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
}

.spacer {
  flex: 1;
}

.merge-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
  padding: 6px 10px;
  border: 1px solid var(--accent);
  border-left-width: 4px;
  border-radius: 6px;
  background: var(--info-bg);
}

.merge-banner.conflicts {
  border-color: var(--danger);
  background: var(--danger-bg);
}

.merge-banner strong {
  color: var(--accent);
}

.merge-banner.conflicts strong {
  color: var(--danger);
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
