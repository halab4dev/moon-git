<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { buildRows, KIND_LETTER, sortRows, type SortKey } from "./fileRows";
import type { RepoStatus } from "../types/status";

const props = defineProps<{
  status: RepoStatus;
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
  <table class="file-table">
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
      <tr v-for="row in rows" :key="row.path" :title="row.origPath ? `${row.origPath} → ${row.path}` : row.path">
        <td class="name">
          <span class="badge" :class="row.kind">{{ KIND_LETTER[row.kind] }}</span>
          {{ row.name }}
        </td>
        <td class="state">{{ row.state }}</td>
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

tbody tr:hover {
  background: var(--bg-sidebar);
}

.badge {
  display: inline-block;
  width: 16px;
  margin-right: 4px;
  text-align: center;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-weight: 700;
}

.badge.added,
.badge.untracked {
  color: var(--success);
}

.badge.modified,
.badge.typeChanged {
  color: var(--warning);
}

.badge.deleted,
.badge.unmerged {
  color: var(--danger);
}

.badge.renamed,
.badge.copied {
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
