<script setup lang="ts">
import type { RepoEntry } from "../types/repo";

defineProps<{
  repo: RepoEntry;
}>();
</script>

<template>
  <section class="repo-view">
    <h1>{{ repo.name }}</h1>
    <p class="muted">{{ repo.path }}</p>
    <p class="muted">Changed files will appear here.</p>
  </section>
</template>

<style scoped>
.repo-view {
  padding: 24px 32px;
}

.muted {
  color: var(--text-muted);
  word-break: break-all;
}
</style>
