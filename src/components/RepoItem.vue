<script setup lang="ts">
import { useRepoStore } from "../stores/repos";
import { showRepoMenu } from "./sidebarMenus";
import AppIcon from "./AppIcon.vue";
import { dropOnPlace, endDrag, isDragged, overDropPlace, startRepoDrag } from "./repoDrag";
import type { RepoEntry } from "../types/repo";

const props = defineProps<{
  repo: RepoEntry;
  depth: number;
}>();

const store = useRepoStore();

// Like Finder (step 35b): click opens one; ⌘-click adds / removes one;
// ⇧-click picks everything from the last clicked one to here.
function onClick(event: MouseEvent) {
  if (event.metaKey || event.ctrlKey) {
    store.togglePicked(props.repo.id);
  } else if (event.shiftKey) {
    store.pickRange(props.repo.id);
  } else {
    store.select(props.repo.id);
  }
}
</script>

<template>
  <li>
    <div
      class="tree-row"
      :class="{
        selected: repo.id === store.selectedId,
        picked: repo.id !== store.selectedId && store.isPicked(repo.id),
        dragging: isDragged('repo', repo.id),
      }"
      :style="{ paddingLeft: `${12 + depth * 14}px` }"
      :title="repo.path"
      draggable="true"
      @click="onClick"
      @contextmenu.prevent.stop="showRepoMenu(repo)"
      @dragstart="startRepoDrag($event, repo)"
      @dragend="endDrag"
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
