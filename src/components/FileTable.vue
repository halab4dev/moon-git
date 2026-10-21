<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  buildRows,
  sortRows,
  stateLabel,
  stateLetter,
  stateTone,
  type FileRow,
  type SortKey,
} from "./fileRows";
import type { RepoStatus } from "../types/status";

const props = defineProps<{
  status: RepoStatus;
}>();

// Paths of the selected rows. Two-way bound: the parent writes
// `v-model:selected="..."` and reads the selection back from it.
const selected = defineModel<string[]>("selected", { required: true });

const emit = defineEmits<{
  /** Right-click on a row; the selection already contains that row. */
  rowMenu: [];
  /** Double-click on a row. */
  rowOpen: [row: FileRow];
}>();

// The chosen sort is remembered for all repos (a viewer convenience only).
const SORT_KEY = "moongit.fileTable.sort";

const COLUMNS: { key: SortKey; label: string }[] = [
  { key: "name", label: "Name" },
  { key: "state", label: "State" },
  { key: "location", label: "Location" },
];

const saved = readSort();
const sortKey = ref<SortKey>(saved.key);
const ascending = ref(saved.ascending);

const rows = computed(() => sortRows(buildRows(props.status), sortKey.value, ascending.value));

const selectedSet = computed(() => new Set(selected.value));
// Row where the last plain or Cmd/Ctrl click happened; Shift-click selects from here.
const anchor = ref<string | null>(null);

// After a refresh some files may be gone (e.g. committed); drop them from the selection.
watch(rows, (current) => {
  const present = new Set(current.map((r) => r.path));
  if (selected.value.some((path) => !present.has(path))) {
    selected.value = selected.value.filter((path) => present.has(path));
  }
});

// Selection like a desktop file list:
// click = only this row, Cmd/Ctrl+click = add/remove this row, Shift+click = range.
function onRowClick(event: MouseEvent, row: FileRow) {
  const paths = rows.value.map((r) => r.path);
  const anchorIndex = anchor.value === null ? -1 : paths.indexOf(anchor.value);

  if (event.shiftKey && anchorIndex !== -1) {
    const rowIndex = paths.indexOf(row.path);
    const [from, to] = anchorIndex < rowIndex ? [anchorIndex, rowIndex] : [rowIndex, anchorIndex];
    selected.value = paths.slice(from, to + 1);
    return;
  }
  if (event.metaKey || event.ctrlKey) {
    selected.value = selectedSet.value.has(row.path)
      ? selected.value.filter((path) => path !== row.path)
      : [...selected.value, row.path];
  } else {
    selected.value = [row.path];
  }
  anchor.value = row.path;
}

function onRowContextMenu(row: FileRow) {
  // Right-click outside the selection works on that row alone, like Finder.
  if (!selectedSet.value.has(row.path)) {
    selected.value = [row.path];
    anchor.value = row.path;
  }
  emit("rowMenu");
}

function selectAll() {
  selected.value = rows.value.map((r) => r.path);
}

// Arrow keys move a single selection up or down, handy for stepping through diffs.
function moveSelection(step: number) {
  const paths = rows.value.map((r) => r.path);
  if (paths.length === 0) {
    return;
  }
  const current = anchor.value === null ? -1 : paths.indexOf(anchor.value);
  const next = current === -1 ? 0 : Math.min(paths.length - 1, Math.max(0, current + step));
  selected.value = [paths[next]];
  anchor.value = paths[next];
  // Keep the newly selected row visible.
  document.querySelector(`tr[data-path="${CSS.escape(paths[next])}"]`)?.scrollIntoView({ block: "nearest" });
}

// Click a header to sort by it; click the same header again to reverse.
function sortBy(key: SortKey) {
  if (sortKey.value === key) {
    ascending.value = !ascending.value;
  } else {
    sortKey.value = key;
    ascending.value = true;
  }
}

watch([sortKey, ascending], () => {
  try {
    localStorage.setItem(SORT_KEY, JSON.stringify({ key: sortKey.value, ascending: ascending.value }));
  } catch {
    // Storage unavailable: the sort just isn't remembered.
  }
});

function readSort(): { key: SortKey; ascending: boolean } {
  try {
    const parsed = JSON.parse(localStorage.getItem(SORT_KEY) ?? "null");
    if (COLUMNS.some((c) => c.key === parsed?.key) && typeof parsed.ascending === "boolean") {
      return parsed;
    }
  } catch {
    // Missing or invalid: fall through to the default.
  }
  return { key: "name", ascending: true };
}
</script>

<template>
  <!-- tabindex makes the table focusable, so it can receive Cmd/Ctrl+A. -->
  <table
    class="file-table"
    tabindex="0"
    @keydown.meta.a.prevent="selectAll"
    @keydown.ctrl.a.prevent="selectAll"
    @keydown.up.prevent="moveSelection(-1)"
    @keydown.down.prevent="moveSelection(1)"
  >
    <thead>
      <tr>
        <th
          v-for="column in COLUMNS"
          :key="column.key"
          :class="column.key"
          :aria-sort="sortKey === column.key ? (ascending ? 'ascending' : 'descending') : 'none'"
        >
          <button type="button" @click="sortBy(column.key)">
            {{ column.label }}
            <span class="arrow">{{ sortKey === column.key ? (ascending ? "▲" : "▼") : "" }}</span>
          </button>
        </th>
      </tr>
    </thead>
    <tbody>
      <tr
        v-for="row in rows"
        :key="row.path"
        :data-path="row.path"
        :class="{ selected: selectedSet.has(row.path) }"
        :title="row.origPath ? `${row.origPath} → ${row.path}` : row.path"
        @click="onRowClick($event, row)"
        @dblclick="emit('rowOpen', row)"
        @contextmenu.prevent="onRowContextMenu(row)"
      >
        <td class="name">
          <span class="badge" :class="stateTone(row.state)">{{ stateLetter(row.state) }}</span>
          {{ row.name }}
        </td>
        <td class="state">{{ stateLabel(row.state) }}</td>
        <td class="location">
          {{ row.location }}
          <span v-if="row.origPath" class="orig">← {{ row.origPath }}</span>
        </td>
      </tr>
    </tbody>
  </table>
</template>

<style scoped>
.file-table {
  width: 100%;
  border-collapse: collapse;
  /* Fixed layout: column widths come from <th>, long paths are cut with "…". */
  table-layout: fixed;
}

th {
  position: sticky;
  top: 0;
  background: var(--bg);
  border-bottom: 1px solid var(--border);
  padding: 0;
  text-align: left;
}

th.name {
  width: 38%;
}

th.state {
  width: 24%;
}

th button {
  width: 100%;
  border: none;
  border-radius: 0;
  background: transparent;
  padding: 5px 8px;
  text-align: left;
  font-weight: 600;
  color: var(--text-muted);
}

th button:hover {
  background: var(--bg-sidebar);
  color: var(--text);
}

.arrow {
  font-size: 9px;
  margin-left: 2px;
}

td {
  padding: 3px 8px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-table:focus {
  outline: none;
}

tbody tr {
  cursor: default;
  /* Shift-click would otherwise also select the text between the rows. */
  user-select: none;
}

tbody tr:hover {
  background: var(--bg-sidebar);
}

tbody tr.selected {
  background: var(--accent);
  color: #fff;
}

tbody tr.selected td,
tbody tr.selected .orig,
tbody tr.selected .badge {
  color: #fff;
}

.badge {
  display: inline-block;
  width: 16px;
  margin-right: 4px;
  text-align: center;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-weight: 700;
}

/* Green = staged for the next commit, amber = changed but not staged. */
.badge.staged,
.badge.untracked {
  color: var(--success);
}

.badge.modified {
  color: var(--warning);
}

.badge.removed,
.badge.conflict {
  color: var(--danger);
}

.badge.renamed {
  color: var(--accent);
}

.badge.untracked {
  opacity: 0.7;
}

.state,
.location,
.orig {
  color: var(--text-muted);
}
</style>
