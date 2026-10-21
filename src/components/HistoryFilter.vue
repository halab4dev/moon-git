<script setup lang="ts">
import { computed } from "vue";
import type { FilterMode } from "../types/log";

// Filter box of the History tab: text (message or commit id prefix), mode
// switch, match count and next/previous buttons (highlight mode).
const props = defineProps<{
  /** Number of matches in the whole history; null while searching or without a filter. */
  matchCount: number | null;
  searching: boolean;
}>();

const text = defineModel<string>("text", { required: true });
const mode = defineModel<FilterMode>("mode", { required: true });

const emit = defineEmits<{
  /** Jump to the next (+1) or previous (-1) match. */
  jump: [step: 1 | -1];
}>();

const active = computed(() => text.value.trim() !== "");

const status = computed(() => {
  if (!active.value) return "";
  if (props.searching) return "Searching…";
  if (props.matchCount === null) return "";
  return props.matchCount === 0 ? "No matches" : `${props.matchCount} match${props.matchCount === 1 ? "" : "es"}`;
});
</script>

<template>
  <div class="history-filter">
    <input
      v-model="text"
      class="text"
      :class="{ active }"
      placeholder="Filter: message or commit id"
      spellcheck="false"
      @keydown.enter.exact.prevent="emit('jump', 1)"
      @keydown.shift.enter.prevent="emit('jump', -1)"
      @keydown.esc="text = ''"
    />
    <div class="modes" role="group" aria-label="Filter mode">
      <button
        :class="{ on: mode === 'highlight' }"
        title="Keep every commit and dim the ones that don't match"
        @click="mode = 'highlight'"
      >
        Highlight
      </button>
      <button :class="{ on: mode === 'hide' }" title="Show matching commits only" @click="mode = 'hide'">Hide others</button>
    </div>
    <template v-if="active && mode === 'highlight'">
      <button class="icon-btn" title="Previous match (Shift+Enter)" :disabled="!matchCount" @click="emit('jump', -1)">↑</button>
      <button class="icon-btn" title="Next match (Enter)" :disabled="!matchCount" @click="emit('jump', 1)">↓</button>
    </template>
    <span class="muted">{{ status }}</span>
  </div>
</template>

<style scoped>
.history-filter {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.text {
  width: 240px;
}

.text.active {
  border-color: var(--accent);
}

.modes {
  display: inline-flex;
}

.modes button {
  border-radius: 0;
}

.modes button:first-child {
  border-radius: 4px 0 0 4px;
}

.modes button:last-child {
  border-left: none;
  border-radius: 0 4px 4px 0;
}

.modes button.on {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}

.muted {
  color: var(--text-muted);
  white-space: nowrap;
}
</style>
