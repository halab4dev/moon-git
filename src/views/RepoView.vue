<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { Menu } from "@tauri-apps/api/menu";
import CommitBox from "../components/CommitBox.vue";
import DiffPanel from "../components/DiffPanel.vue";
import FileTable from "../components/FileTable.vue";
import { buildRows, type FileRow } from "../components/fileRows";
import { openDiffWindow } from "../windows";
import { getStatus } from "../api/status";
import { stageFiles, unstageFiles } from "../api/stage";
import { commitChanges } from "../api/commit";
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
const error = ref<AppError | null>(null);
const selectedPaths = ref<string[]>([]);
const commitMessage = ref("");
const amend = ref(false);

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
  if (busy.value) {
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
  if (await run(() => commitChanges(props.repo.id, commitMessage.value, amend.value))) {
    commitMessage.value = "";
    amend.value = false;
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

// ---- Splitter between the file table (top) and the diff (bottom) ----

const RATIO_KEY = "moongit.repoView.filesRatio";
const split = ref<HTMLDivElement | null>(null);
// Share of the height used by the file table.
const filesRatio = ref(readRatio());

function startResize(event: MouseEvent) {
  const box = split.value?.getBoundingClientRect();
  if (!box) {
    return;
  }
  event.preventDefault(); // don't start selecting text while dragging
  const onMove = (move: MouseEvent) => {
    const ratio = (move.clientY - box.top) / box.height;
    filesRatio.value = Math.min(0.85, Math.max(0.15, ratio));
  };
  const onUp = () => {
    window.removeEventListener("mousemove", onMove);
    try {
      localStorage.setItem(RATIO_KEY, String(filesRatio.value));
    } catch {
      // Not remembered; fine.
    }
  };
  window.addEventListener("mousemove", onMove);
  window.addEventListener("mouseup", onUp, { once: true });
}

function readRatio(): number {
  try {
    const value = Number(localStorage.getItem(RATIO_KEY));
    if (value >= 0.15 && value <= 0.85) {
      return value;
    }
  } catch {
    // Use the default.
  }
  return 0.35;
}

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
    </header>

    <div class="toolbar">
      <button :disabled="busy || pathsToStage.length === 0" title="Stage selected files" @click="stageSelected">
        Stage
      </button>
      <button :disabled="busy || pathsToUnstage.length === 0" title="Unstage selected files" @click="unstageSelected">
        Unstage
      </button>
      <span class="spacer"></span>
      <button :disabled="busy" title="Reload changed files" @click="refresh">Refresh</button>
    </div>

    <p v-if="error" class="err message">{{ error.message }}</p>

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
      :repo-id="repo.id"
      :staged-count="stagedCount"
      :busy="busy"
      @commit="commitStaged"
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

.muted {
  color: var(--text-muted);
}

.message {
  white-space: pre-wrap;
  word-break: break-word;
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
