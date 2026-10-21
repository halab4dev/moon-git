<script setup lang="ts">
import { useRepoStore } from "../stores/repos";
import { showAddMenu } from "./sidebarMenus";
import GroupItem from "./GroupItem.vue";
import RepoItem from "./RepoItem.vue";

const store = useRepoStore();
</script>

<template>
  <aside class="sidebar">
    <header class="sidebar-header">
      <span>Repositories</span>
      <button class="icon-btn" title="Add repository or group" @click="showAddMenu(null)">+</button>
    </header>

    <div v-if="store.error" class="error">
      <span>{{ store.error.message }}</span>
      <button class="icon-btn" title="Dismiss" @click="store.clearError()">×</button>
    </div>

    <!-- Right-click on empty space opens the same menu as "+". -->
    <div class="sidebar-body" @contextmenu.prevent="showAddMenu(null)">
      <ul v-if="!store.isEmpty" class="tree">
        <GroupItem v-for="node in store.tree.groups" :key="node.group.id" :node="node" :depth="0" />
        <RepoItem v-for="repo in store.tree.repos" :key="repo.id" :repo="repo" :depth="0" />
      </ul>
      <p v-else class="empty">No repositories yet. Click + to add one.</p>
    </div>
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

.sidebar-body {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.empty {
  padding: 4px 12px;
  margin: 0;
  color: var(--text-muted);
}
</style>
