<script setup lang="ts">
import { computed, defineAsyncComponent, ref, watch } from "vue";
import { getFileDiff } from "../api/diff";
import { toAppError } from "../api/errors";
import type { FileRow } from "./fileRows";
import type { AppError } from "../types/error";
import type { DiffMode, FileDiff } from "../types/diff";

// Monaco is large: load it the first time a diff is shown, not at app start.
const DiffEditor = defineAsyncComponent(() => import("./DiffEditor.vue"));

const props = defineProps<{
  repoId: string;
  /** The file to show, or null when zero or several files are selected. */
  row: FileRow | null;
  selectedCount: number;
  /** Changes whenever the status was reloaded, so the diff reloads too. */
  version: number;
}>();

const OPTIONS_KEY = "moongit.diff.options";
const saved = readOptions();
const sideBySide = ref(saved.sideBySide);
const ignoreWhitespace = ref(saved.ignoreWhitespace);

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

watch([sideBySide, ignoreWhitespace], () => {
  try {
    localStorage.setItem(
      OPTIONS_KEY,
      JSON.stringify({ sideBySide: sideBySide.value, ignoreWhitespace: ignoreWhitespace.value }),
    );
  } catch {
    // Storage unavailable: options just aren't remembered.
  }
});

function readOptions(): { sideBySide: boolean; ignoreWhitespace: boolean } {
  const defaults = { sideBySide: true, ignoreWhitespace: false };
  try {
    const parsed = JSON.parse(localStorage.getItem(OPTIONS_KEY) ?? "null");
    if (typeof parsed?.sideBySide === "boolean" && typeof parsed?.ignoreWhitespace === "boolean") {
      return parsed;
    }
  } catch {
    // Missing or invalid: use the defaults.
  }
  return defaults;
}
</script>

<template>
  <section class="diff-panel">
    <header v-if="row" class="diff-header">
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
      <span v-if="diff" class="labels">{{ diff.oldLabel }} ↔ {{ diff.newLabel }}</span>
      <span class="spacer"></span>
      <label><input v-model="sideBySide" type="checkbox" /> Side by side</label>
      <label><input v-model="ignoreWhitespace" type="checkbox" /> Ignore whitespace</label>
    </header>

    <div class="diff-body">
      <p v-if="row === null" class="message">
        {{ selectedCount > 1 ? `${selectedCount} files selected.` : "Select a file to see its changes." }}
      </p>
      <p v-else-if="error" class="message err">{{ error.message }}</p>
      <p v-else-if="diff?.binary" class="message">Binary file — no text to compare.</p>
      <p v-else-if="diff?.tooLarge" class="message">File is larger than 5 MB — not shown.</p>
      <DiffEditor
        v-else-if="diff"
        :path="row.path"
        :old-text="diff.oldText ?? ''"
        :new-text="diff.newText ?? ''"
        :side-by-side="sideBySide"
        :ignore-whitespace="ignoreWhitespace"
      />
      <p v-else-if="loading" class="message">Loading…</p>
    </div>
  </section>
</template>

<style scoped>
.diff-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  border-top: 1px solid var(--border);
}

.diff-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 0;
  white-space: nowrap;
  overflow: hidden;
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

.labels {
  color: var(--text-muted);
}

.spacer {
  flex: 1;
}

label {
  display: flex;
  align-items: center;
  gap: 4px;
  color: var(--text-muted);
}

.diff-body {
  flex: 1;
  min-height: 0;
}

.message {
  margin: 12px 0;
  color: var(--text-muted);
}

.message.err {
  color: var(--danger);
  white-space: pre-wrap;
}
</style>
