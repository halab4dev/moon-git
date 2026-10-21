<script setup lang="ts">
import { defineAsyncComponent, ref, watch } from "vue";
import type { AppError } from "../types/error";
import type { FileDiff } from "../types/diff";

// Shows one loaded FileDiff: the header with the "Side by side" / "Ignore
// whitespace" options (remembered), and Monaco or a message below.
// Used by DiffPanel (working tree, step 8) and CommitDetails (history, step 18);
// each puts its own buttons into the "controls" slot.

// Monaco is large: load it the first time a diff is shown, not at app start.
const DiffEditor = defineAsyncComponent(() => import("./DiffEditor.vue"));

defineProps<{
  /** The file shown; null = nothing chosen (the placeholder is shown). */
  path: string | null;
  diff: FileDiff | null;
  loading: boolean;
  error: AppError | null;
  placeholder: string;
}>();

const OPTIONS_KEY = "moongit.diff.options";
const saved = readOptions();
const sideBySide = ref(saved.sideBySide);
const ignoreWhitespace = ref(saved.ignoreWhitespace);

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
  <section class="diff-view">
    <header v-if="path !== null" class="diff-header">
      <!-- Filled by the parent component, e.g. the Working Tree / Staged buttons. -->
      <slot name="controls"></slot>
      <span v-if="diff" class="labels">{{ diff.oldLabel }} ↔ {{ diff.newLabel }}</span>
      <span class="spacer"></span>
      <label><input v-model="sideBySide" type="checkbox" /> Side by side</label>
      <label><input v-model="ignoreWhitespace" type="checkbox" /> Ignore whitespace</label>
    </header>

    <div class="diff-body">
      <p v-if="path === null" class="message">{{ placeholder }}</p>
      <p v-else-if="error" class="message err">{{ error.message }}</p>
      <p v-else-if="diff?.binary" class="message">Binary file — no text to compare.</p>
      <p v-else-if="diff?.tooLarge" class="message">File is larger than 5 MB — not shown.</p>
      <DiffEditor
        v-else-if="diff"
        :path="path"
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
.diff-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-width: 0;
}

.diff-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 0;
  white-space: nowrap;
  overflow: hidden;
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
