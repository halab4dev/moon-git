<script setup lang="ts">
import { onMounted, ref, shallowRef, watch } from "vue";
import CommitList from "../components/CommitList.vue";
import RefFilter from "../components/RefFilter.vue";
import { currentBranchRefs } from "../components/refSelection";
import { LaneLayout, type GraphRow } from "../graph/laneLayout";
import { getLog, listRefs } from "../api/log";
import { toAppError } from "../api/errors";
import type { AppError } from "../types/error";
import type { CommitSummary, RefInfo } from "../types/log";

// "History" tab: all branches, remote branches and tags, newest first,
// loaded page by page while scrolling.
const props = defineProps<{
  repoId: string;
  /** Changes whenever the status was reloaded: new commits may exist. */
  version: number;
}>();

const PAGE_SIZE = 500;
const MAX_PAGE = 2000; // limit of the Rust side

// shallowRef: Vue only watches the array as a whole, not every field of every
// commit. With tens of thousands of commits, deep watching would be slow.
// Consequence: always replace the array, never push into it.
const commits = shallowRef<CommitSummary[]>([]);
const hasMore = ref(false);
const loading = ref(false);
const error = ref<AppError | null>(null);
const selected = ref<string | null>(null);

// ---- which refs to show (remembered per repository) ----

const refs = ref<RefInfo[]>([]);
// null = all. Stale names (deleted branches) are simply ignored by Rust.
// undefined from readSelection() = never chosen for this repository yet.
const saved = readSelection();
const refSelection = ref<string[] | null>(saved ?? null);
// False until there is a choice to remember: the user's, or the
// first-time default (the current branch), set once the refs are known.
let decided = saved !== undefined;

function selectionKey(): string {
  return `moongit.history.refs.${props.repoId}`;
}

function readSelection(): string[] | null | undefined {
  try {
    const stored = localStorage.getItem(selectionKey());
    if (stored === null) {
      return undefined;
    }
    const parsed = JSON.parse(stored);
    if (parsed === null) {
      return null; // "All" was chosen
    }
    if (Array.isArray(parsed) && parsed.every((name) => typeof name === "string")) {
      return parsed;
    }
  } catch {
    // Unreadable: treat as never chosen.
  }
  return undefined;
}

watch(refSelection, (current) => {
  decided = true;
  try {
    localStorage.setItem(selectionKey(), JSON.stringify(current));
  } catch {
    // Not remembered; fine.
  }
  reload();
});

async function loadRefs() {
  try {
    refs.value = await listRefs(props.repoId);
  } catch (err) {
    error.value = toAppError(err);
  }
}

// The graph is laid out page by page, like the commits are loaded. The
// LaneLayout object keeps its own state between pages and is not reactive;
// graphRows (a copy of its rows) is what the template sees.
let layout = new LaneLayout();
const graphRows = shallowRef<GraphRow[]>([]);
const graphLanes = ref(0);

function showGraph() {
  graphRows.value = layout.rows.slice(); // new array, so the shallowRef notices
  graphLanes.value = layout.maxLanes;
}

// Bumped by every reload; an answer for an older generation is dropped
// (e.g. a page requested just before the list was reloaded).
let generation = 0;

async function reload() {
  const current = ++generation;
  loading.value = true;
  // Keep about as many commits as are loaded now, so the scroll position survives.
  const count = Math.min(MAX_PAGE, Math.max(PAGE_SIZE, commits.value.length));
  try {
    const page = await getLog(props.repoId, 0, count, refSelection.value);
    if (current !== generation) {
      return;
    }
    commits.value = page.commits;
    hasMore.value = page.hasMore;
    error.value = null;
    layout = new LaneLayout();
    layout.add(page.commits);
    showGraph();
  } catch (err) {
    if (current === generation) {
      error.value = toAppError(err);
    }
  } finally {
    if (current === generation) {
      loading.value = false;
    }
  }
}

async function loadMore() {
  if (loading.value || !hasMore.value) {
    return;
  }
  const current = generation;
  loading.value = true;
  try {
    const page = await getLog(props.repoId, commits.value.length, PAGE_SIZE, refSelection.value);
    if (current !== generation) {
      return;
    }
    commits.value = commits.value.concat(page.commits);
    hasMore.value = page.hasMore;
    layout.add(page.commits); // continues where the previous page ended
    showGraph();
  } catch (err) {
    if (current === generation) {
      error.value = toAppError(err);
    }
  } finally {
    if (current === generation) {
      loading.value = false;
    }
  }
}

async function reloadAll() {
  await loadRefs();
  // First visit: start with the current branch instead of everything.
  // (Assigning triggers the watch above, which saves and reloads.)
  if (!decided) {
    const current = currentBranchRefs(refs.value);
    if (current.length > 0) {
      refSelection.value = current;
      return;
    }
    // No commits yet: nothing to choose from; decide on a later visit.
  }
  reload();
}

onMounted(reloadAll);
watch(() => props.version, reloadAll);
</script>

<template>
  <div class="history">
    <div class="history-toolbar">
      <RefFilter v-model:selection="refSelection" :refs="refs" />
    </div>
    <p v-if="error" class="err message">{{ error.message }}</p>
    <p v-if="refSelection !== null && refSelection.length === 0" class="muted">
      No branches or tags selected. Choose some with the button above.
    </p>
    <CommitList
      v-model:selected="selected"
      :commits="commits"
      :graph="graphRows"
      :graph-lanes="graphLanes"
      :has-more="hasMore"
      :loading="loading"
      @load-more="loadMore"
    />
    <p class="muted footer">
      {{ commits.length }} commit(s) loaded{{ hasMore ? ", more below" : "" }}{{ loading ? " · loading…" : "" }}
    </p>
  </div>
</template>

<style scoped>
.history {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.history-toolbar {
  display: flex;
  gap: 8px;
  margin-bottom: 6px;
}

.footer {
  margin: 4px 0 0;
  font-size: 12px;
}

.muted {
  color: var(--text-muted);
}

.message {
  white-space: pre-wrap;
}
</style>
