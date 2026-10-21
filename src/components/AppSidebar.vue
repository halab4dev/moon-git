<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog";
import { useRepoStore } from "../stores/repos";

const store = useRepoStore();

async function onAdd() {
  const picked = await open({ directory: true, multiple: false, title: "Add repository" });
  // null means the user cancelled the dialog.
  if (typeof picked === "string") {
    await store.add(picked);
  }
}
</script>

<template>
  <aside class="sidebar">
    <header class="sidebar-header">
      <span>Repositories</span>
      <button class="icon-btn" title="Add repository" @click="onAdd">+</button>
    </header>

    <div v-if="store.error" class="error">
      <span>{{ store.error.message }}</span>
      <button class="icon-btn" title="Dismiss" @click="store.clearError()">×</button>
    </div>

    <ul v-if="store.sortedRepos.length > 0" class="repo-list">
      <li
        v-for="repo in store.sortedRepos"
        :key="repo.id"
        :class="{ selected: repo.id === store.selectedId }"
        :title="repo.path"
        @click="store.select(repo.id)"
      >
        <span class="repo-name">{{ repo.name }}</span>
        <!-- .stop: don't let this click also select the row. -->
        <button
          class="icon-btn remove"
          title="Remove from list (files on disk are kept)"
          @click.stop="store.remove(repo.id)"
        >
          ×
        </button>
      </li>
    </ul>
    <p v-else class="empty">No repositories yet. Click + to add one.</p>
  </aside>
</template>

<style scoped>
.sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border);
  overflow: hidden;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px 6px 12px;
  font-weight: 600;
  text-transform: uppercase;
  font-size: 11px;
  letter-spacing: 0.05em;
  color: var(--text-muted);
  border-bottom: 1px solid var(--border);
}

.icon-btn {
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 16px;
  line-height: 1;
  padding: 2px 6px;
  border-radius: 4px;
  cursor: pointer;
}

.icon-btn:hover {
  background: var(--border);
  color: var(--text);
}

.error {
  display: flex;
  align-items: flex-start;
  gap: 4px;
  margin: 8px;
  padding: 6px 8px;
  border: 1px solid var(--danger);
  border-radius: 4px;
  color: var(--danger);
  word-break: break-word;
}

.error span {
  flex: 1;
}

.repo-list {
  flex: 1;
  overflow-y: auto;
  list-style: none;
  margin: 0;
  padding: 4px 0;
}

.repo-list li {
  display: flex;
  align-items: center;
  padding: 4px 8px 4px 12px;
  cursor: pointer;
}

.repo-list li:hover {
  background: var(--border);
}

.repo-list li.selected {
  background: var(--accent);
  color: #fff;
}

.repo-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Only show the remove button on the row under the mouse. */
.remove {
  visibility: hidden;
}

.repo-list li:hover .remove {
  visibility: visible;
}

.repo-list li.selected .remove {
  color: #fff;
}

.empty {
  padding: 8px 12px;
  color: var(--text-muted);
}
</style>
