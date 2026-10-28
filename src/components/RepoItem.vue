<script setup lang="ts">
import { useRepoStore } from "../stores/repos";
import { showRepoMenu } from "./sidebarMenus";
import AppIcon from "./AppIcon.vue";
import { draggedRepo, dropOnPlace, endRepoDrag, overDropPlace, startRepoDrag } from "./repoDrag";
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
      :class="{ selected: repo.id === store.selectedId, dragging: draggedRepo?.id === repo.id }"
      :style="{ paddingLeft: `${12 + depth * 14}px` }"
      :title="repo.path"
      draggable="true"
      @click="store.select(repo.id)"
      @contextmenu.prevent.stop="showRepoMenu(repo)"
      @dragstart="startRepoDrag($event, repo)"
      @dragend="endRepoDrag"
      @dragover="overDropPlace($event, repo.groupId)"
      @drop="dropOnPlace($event, repo.groupId)"
    >
      <!-- Empty, under the groups' ▸/▾, so repo and folder icons line up. -->
      <span class="tree-icon"></span>
      <AppIcon name="repo" />
      <span class="tree-name">{{ repo.name }}</span>
    </div>
  </li>
</template>
