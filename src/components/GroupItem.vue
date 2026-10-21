<script setup lang="ts">
import { useRepoStore } from "../stores/repos";
import { showGroupMenu } from "./sidebarMenus";
import RepoItem from "./RepoItem.vue";
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
      <!-- Inline SVG instead of an emoji: looks the same on every OS and follows the theme color. -->
      <svg class="folder-icon" viewBox="0 0 16 16" aria-hidden="true">
        <path
          v-if="store.isCollapsed(node.group.id)"
          d="M1.5 3.5a1 1 0 0 1 1-1h3.6l1.5 1.5h5.9a1 1 0 0 1 1 1v7.5a1 1 0 0 1-1 1h-11a1 1 0 0 1-1-1z"
        />
        <path
          v-else
          d="M1.5 3.5a1 1 0 0 1 1-1h3.6l1.5 1.5h5.4a1 1 0 0 1 1 1v1.5H4.2a1 1 0 0 0-.95.68L1.5 12.5zM3.3 7.5h11.4a.6.6 0 0 1 .57.8l-1.5 4.5a1 1 0 0 1-.95.7H1.9z"
        />
      </svg>
      <span class="tree-name">{{ node.group.name }}</span>
    </div>
    <ul v-if="!store.isCollapsed(node.group.id)" class="tree">
      <!-- A component can use itself by its file name: this is what makes the tree recursive. -->
      <GroupItem v-for="child in node.children" :key="child.group.id" :node="child" :depth="depth + 1" />
      <RepoItem v-for="repo in node.repos" :key="repo.id" :repo="repo" :depth="depth + 1" />
    </ul>
  </li>
</template>
