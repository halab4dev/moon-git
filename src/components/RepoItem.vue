<script setup lang="ts">
import { useRepoStore } from "../stores/repos";
import { showRepoMenu } from "./sidebarMenus";
import type { RepoEntry } from "../types/repo";

defineProps<{
  repo: RepoEntry;
  depth: number;
}>();

const store = useRepoStore();
</script>

<template>
  <li>
    <div
      class="tree-row"
      :class="{ selected: repo.id === store.selectedId }"
      :style="{ paddingLeft: `${12 + depth * 14}px` }"
      :title="repo.path"
      @click="store.select(repo.id)"
      @contextmenu.prevent.stop="showRepoMenu(repo)"
    >
      <span class="tree-icon">◆</span>
      <span class="tree-name">{{ repo.name }}</span>
    </div>
  </li>
</template>
