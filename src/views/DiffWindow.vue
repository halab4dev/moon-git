<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import DiffPanel from "../components/DiffPanel.vue";
import { buildRows } from "../components/fileRows";
import { getStatus } from "../api/status";
import { toAppError } from "../api/errors";
import type { AppError } from "../types/error";
import type { RepoStatus } from "../types/status";

// Root of a separate diff window (see windows.ts). Which file to show comes
// from the URL, e.g. index.html?view=diff&repoId=...&path=src/main.ts
const params = new URLSearchParams(window.location.search);
const repoId = params.get("repoId") ?? "";
const path = params.get("path") ?? "";

const status = ref<RepoStatus | null>(null);
const version = ref(0);
const error = ref<AppError | null>(null);

// Read the file's state from a fresh status (not from the URL), so the
// Working Tree / Staged choice stays right after staging in the main window.
const row = computed(() => {
  if (status.value === null) {
    return null;
  }
  return buildRows(status.value).find((r) => r.path === path) ?? null;
});

let loading = false;

async function refresh() {
  if (loading) {
    return;
  }
  loading = true;
  try {
    status.value = await getStatus(repoId);
    version.value++;
    error.value = null;
  } catch (err) {
    error.value = toAppError(err);
  } finally {
    loading = false;
  }
}

onMounted(() => {
  refresh();
  window.addEventListener("focus", refresh);
});

onUnmounted(() => {
  window.removeEventListener("focus", refresh);
});
</script>

<template>
  <div class="diff-window">
    <header>
      <strong>{{ path.slice(path.lastIndexOf("/") + 1) }}</strong>
      <span class="muted">{{ path }}</span>
    </header>
    <p v-if="error" class="err message">{{ error.message }}</p>
    <p v-else-if="status !== null && row === null" class="muted message">
      This file has no changes any more (committed, discarded or removed).
    </p>
    <div v-if="row" class="body">
      <DiffPanel :repo-id="repoId" :row="row" :selected-count="1" :version="version" />
    </div>
  </div>
</template>

<style scoped>
.diff-window {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 8px 12px;
}

header {
  display: flex;
  align-items: baseline;
  gap: 10px;
  padding-bottom: 6px;
  white-space: nowrap;
  overflow: hidden;
}

.muted {
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
}

.message {
  margin: 12px 0;
  white-space: pre-wrap;
}

.body {
  flex: 1;
  min-height: 0;
}
</style>
