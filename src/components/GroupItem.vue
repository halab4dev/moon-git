<script setup lang="ts">
import { useRepoStore } from "../stores/repos";
import { showGroupMenu } from "./sidebarMenus";
import RepoItem from "./RepoItem.vue";
import AppIcon from "./AppIcon.vue";
import type { GroupNode } from "../stores/repoTree";

defineProps<{
  node: GroupNode;
  depth: number;
}>();

const store = useRepoStore();
</script>

<template>
  <li>
    <div
      class="tree-row group"
      :style="{ paddingLeft: `${12 + depth * 14}px` }"
      @click="store.toggleCollapsed(node.group.id)"
      @contextmenu.prevent.stop="showGroupMenu(node.group)"
    >
      <span class="tree-icon">{{ store.isCollapsed(node.group.id) ? "▸" : "▾" }}</span>
      <AppIcon :name="store.isCollapsed(node.group.id) ? 'folder' : 'folderOpen'" />
      <span class="tree-name">{{ node.group.name }}</span>
    </div>
    <ul v-if="!store.isCollapsed(node.group.id)" class="tree">
      <!-- A component can use itself by its file name: this is what makes the tree recursive. -->
      <GroupItem v-for="child in node.children" :key="child.group.id" :node="child" :depth="depth + 1" />
      <RepoItem v-for="repo in node.repos" :key="repo.id" :repo="repo" :depth="depth + 1" />
    </ul>
  </li>
</template>
