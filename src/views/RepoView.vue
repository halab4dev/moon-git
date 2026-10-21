<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { Menu } from "@tauri-apps/api/menu";
import FileTable from "../components/FileTable.vue";
import { buildRows } from "../components/fileRows";
import { getStatus } from "../api/status";
import { stageFiles, unstageFiles } from "../api/stage";
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
const busy = ref(false);
const error = ref<AppError | null>(null);
const selectedPaths = ref<string[]>([]);

const rows = computed(() => (status.value === null ? [] : buildRows(status.value)));
const selectedRows = computed(() => {
  const selected = new Set(selectedPaths.value);
  return rows.value.filter((row) => selected.has(row.path));
});

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
async function run(action: () => Promise<RepoStatus>) {
  if (busy.value) {
    return;
  }
  busy.value = true;
  try {
    status.value = await action();
    error.value = null;
  } catch (err) {
    error.value = toAppError(err);
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

async function showRowMenu() {
  const menu = await Menu.new({
    items: [
      { text: "Stage", enabled: pathsToStage.value.length > 0, action: () => void stageSelected() },
      { text: "Unstage", enabled: pathsToUnstage.value.length > 0, action: () => void unstageSelected() },
    ],
  });
  await menu.popup();
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
      <div>
        <h1>{{ repo.name }}</h1>
        <p class="muted path">{{ repo.path }}</p>
      </div>
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

    <template v-if="status">
      <p v-if="rows.length === 0" class="muted">Nothing to commit, working tree clean.</p>
      <FileTable v-else v-model:selected="selectedPaths" :status="status" @row-menu="showRowMenu" />
    </template>
    <p v-else-if="busy" class="muted">Loading…</p>
  </section>
</template>

<style scoped>
.repo-view {
  padding: 20px 28px;
}

.repo-header {
  margin-bottom: 12px;
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
  margin-bottom: 10px;
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
</style>
