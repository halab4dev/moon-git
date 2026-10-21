<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from "vue";
import { currentBranchRefs } from "./refSelection";
import type { RefInfo, RefKind } from "../types/log";

// "Branches ▾" button with a drop-down of checkboxes: which branches,
// remote branches and tags the history shows.
const props = defineProps<{
  refs: RefInfo[];
}>();

/**
 * Full names of the chosen refs, or null for "all" (which also includes
 * branches created later). Always replaced with a new array, never changed in place.
 */
const selection = defineModel<string[] | null>("selection", { required: true });

const open = ref(false);
const filter = ref("");
const root = ref<HTMLElement | null>(null);

const GROUPS: { title: string; kinds: RefKind[] }[] = [
  { title: "Local branches", kinds: ["head", "branch"] },
  { title: "Remote branches", kinds: ["remote"] },
  { title: "Tags", kinds: ["tag"] },
];

const chosen = computed(() => new Set(selection.value ?? props.refs.map((r) => r.fullName)));

// Groups with the refs matching the filter text; empty groups are left out.
const groups = computed(() => {
  const text = filter.value.trim().toLowerCase();
  return GROUPS.map((group) => ({
    title: group.title,
    refs: props.refs.filter((r) => group.kinds.includes(r.kind) && r.name.toLowerCase().includes(text)),
  })).filter((group) => group.refs.length > 0);
});

const label = computed(() => {
  if (selection.value === null) {
    return "All refs";
  }
  const shown = props.refs.filter((r) => chosen.value.has(r.fullName)).length;
  return `${shown} of ${props.refs.length} refs`;
});

// Stores a new choice. Everything ticked again = back to "all".
function choose(names: Set<string>) {
  const all = props.refs.every((r) => names.has(r.fullName));
  selection.value = all ? null : props.refs.filter((r) => names.has(r.fullName)).map((r) => r.fullName);
}

function toggle(info: RefInfo) {
  const names = new Set(chosen.value);
  if (names.has(info.fullName)) {
    names.delete(info.fullName);
  } else {
    names.add(info.fullName);
  }
  choose(names);
}

// Ticks or unticks every (visible) ref of a group.
function setGroup(refs: RefInfo[], on: boolean) {
  const names = new Set(chosen.value);
  for (const info of refs) {
    if (on) {
      names.add(info.fullName);
    } else {
      names.delete(info.fullName);
    }
  }
  choose(names);
}

function showAll() {
  selection.value = null;
}

function showNone() {
  selection.value = [];
}

// The checked-out branch (or detached HEAD) and its upstream.
function showCurrent() {
  selection.value = currentBranchRefs(props.refs);
}

// Close when clicking anywhere outside the drop-down.
function onDocumentMouseDown(event: MouseEvent) {
  if (root.value && !root.value.contains(event.target as Node)) {
    open.value = false;
  }
}

watch(open, (isOpen) => {
  if (isOpen) {
    document.addEventListener("mousedown", onDocumentMouseDown);
  } else {
    document.removeEventListener("mousedown", onDocumentMouseDown);
    filter.value = "";
  }
});
onUnmounted(() => document.removeEventListener("mousedown", onDocumentMouseDown));
</script>

<template>
  <div ref="root" class="ref-filter" @keydown.esc="open = false">
    <button :class="{ active: selection !== null }" @click="open = !open">{{ label }} ▾</button>

    <div v-if="open" class="panel">
      <input v-model="filter" placeholder="Filter branches and tags" spellcheck="false" />
      <div class="presets">
        <button @click="showAll">All</button>
        <button title="The checked-out branch and its upstream" @click="showCurrent">Current branch</button>
        <button @click="showNone">None</button>
      </div>

      <div class="list">
        <section v-for="group in groups" :key="group.title">
          <h3>
            {{ group.title }}
            <span class="group-actions">
              <button class="link" @click="setGroup(group.refs, true)">all</button>
              <button class="link" @click="setGroup(group.refs, false)">none</button>
            </span>
          </h3>
          <label v-for="info in group.refs" :key="info.fullName" class="item" :title="info.fullName">
            <input type="checkbox" :checked="chosen.has(info.fullName)" @change="toggle(info)" />
            <span :class="['name', info.kind, { current: info.current }]">{{ info.name }}</span>
          </label>
        </section>
        <p v-if="groups.length === 0" class="muted">No matching refs.</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.ref-filter {
  position: relative;
}

button.active {
  border-color: var(--accent);
  color: var(--accent);
}

.panel {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  z-index: 10;
  width: 300px;
  padding: 8px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg);
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.18);
}

.panel input:not([type="checkbox"]) {
  width: 100%;
}

.presets {
  display: flex;
  gap: 6px;
  margin: 8px 0;
}

.list {
  max-height: 360px;
  overflow-y: auto;
}

h3 {
  display: flex;
  justify-content: space-between;
  margin: 8px 0 2px;
  font-size: 11px;
  text-transform: uppercase;
  color: var(--text-muted);
}

.group-actions {
  display: flex;
  gap: 6px;
  text-transform: none;
}

button.link {
  border: none;
  background: none;
  padding: 0;
  color: var(--accent);
  font-size: 11px;
}

.item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 2px 0;
}

.name {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

/* Same colors as the labels in the history. */
.name.branch {
  color: var(--success);
}

.name.remote {
  color: var(--accent);
}

.name.tag {
  color: var(--warning);
}

.name.head {
  color: var(--danger);
}

.name.current {
  font-weight: 700;
}

.muted {
  color: var(--text-muted);
}
</style>
