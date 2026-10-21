<script setup lang="ts">
import { computed, ref, watch } from "vue";
import DiffView from "./DiffView.vue";
import { getFileDiff } from "../api/diff";
import { toAppError } from "../api/errors";
import type { FileRow } from "./fileRows";
import type { AppError } from "../types/error";
import type { DiffMode, FileDiff } from "../types/diff";

// Diff of a changed file in the Changes tab. Decides which comparison to show
// (Working Tree / Staged / Conflict) and loads it; DiffView draws it.
const props = defineProps<{
  repoId: string;
  /** The file to show, or null when zero or several files are selected. */
  row: FileRow | null;
  selectedCount: number;
  /** Changes whenever the status was reloaded, so the diff reloads too. */
  version: number;
}>();

const diff = ref<FileDiff | null>(null);
const loading = ref(false);
const error = ref<AppError | null>(null);

const MODE_LABEL: Record<DiffMode, string> = {
  unstaged: "Working Tree",
  staged: "Staged",
  conflict: "Conflict",
};

// Which comparisons make sense for this file. A "Staged Modified" file has
// both: what is staged, and what changed after staging.
const modes = computed<DiffMode[]>(() => {
  const row = props.row;
  if (row === null) {
    return [];
  }
  if (row.state === "conflicted") {
    return ["conflict"];
  }
  const list: DiffMode[] = [];
  if (row.hasUnstaged) {
    list.push("unstaged");
  }
  if (row.hasStaged) {
    list.push("staged");
  }
  return list;
});

const mode = ref<DiffMode>("unstaged");

// New file selected: start with its first comparison. Same file but that
// comparison disappeared (e.g. everything got staged): switch to one that exists.
watch(
  () => [props.row?.path, modes.value] as const,
  ([path, list], old) => {
    const fileChanged = old === undefined || old[0] !== path;
    if (list.length > 0 && (fileChanged || !list.includes(mode.value))) {
      mode.value = list[0];
    }
  },
  { immediate: true },
);

// Each load gets a number; an answer that arrives after a newer request
// started is ignored, so clicking quickly through files never shows a stale diff.
let lastRequest = 0;

async function load() {
  const row = props.row;
  if (row === null) {
    diff.value = null;
    error.value = null;
    return;
  }
  const request = ++lastRequest;
  loading.value = true;
  try {
    const origPath = mode.value === "staged" ? row.origPath : null;
    const result = await getFileDiff(props.repoId, row.path, origPath, mode.value);
    if (request === lastRequest) {
      diff.value = result;
      error.value = null;
    }
  } catch (err) {
    if (request === lastRequest) {
      diff.value = null;
      error.value = toAppError(err);
    }
  } finally {
    if (request === lastRequest) {
      loading.value = false;
    }
  }
}

watch(() => [props.row?.path, mode.value, props.version], load, { immediate: true });
</script>

<template>
  <DiffView
    class="diff-panel"
    :path="row?.path ?? null"
    :diff="diff"
    :loading="loading"
    :error="error"
    :placeholder="selectedCount > 1 ? `${selectedCount} files selected.` : 'Select a file to see its changes.'"
  >
    <template #controls>
      <div class="modes">
        <button
          v-for="m in modes"
          :key="m"
          type="button"
          :class="{ active: m === mode }"
          :disabled="modes.length === 1"
          @click="mode = m"
        >
          {{ MODE_LABEL[m] }}
        </button>
      </div>
    </template>
  </DiffView>
</template>

<style scoped>
.diff-panel {
  border-top: 1px solid var(--border);
}

.modes {
  display: flex;
}

.modes button {
  border-radius: 0;
  padding: 2px 10px;
}

.modes button:first-child {
  border-radius: 4px 0 0 4px;
}

.modes button:last-child {
  border-radius: 0 4px 4px 0;
}

.modes button + button {
  border-left: none;
}

.modes button.active {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
  opacity: 1;
}
</style>
