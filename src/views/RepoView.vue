<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import FileTable from "../components/FileTable.vue";
import { getStatus } from "../api/status";
import { toAppError } from "../api/errors";
import type { AppError } from "../types/error";
import type { RepoEntry } from "../types/repo";
import type { RepoStatus } from "../types/status";

const props = defineProps<{
  repo: RepoEntry;
}>();

// Kept in this view, not in a store: App.vue rebuilds RepoView (via :key)
// for every repo, so each repo starts with fresh state.
const status = ref<RepoStatus | null>(null);
const loading = ref(false);
const error = ref<AppError | null>(null);

const changeCount = computed(() => {
  const s = status.value;
  return s === null ? 0 : s.conflicted.length + s.staged.length + s.unstaged.length + s.untracked.length;
});

async function refresh() {
  // Focus events can arrive quickly one after another; one refresh at a time.
  if (loading.value) {
    return;
  }
  loading.value = true;
  try {
    status.value = await getStatus(props.repo.id);
    error.value = null;
  } catch (err) {
    error.value = toAppError(err);
  } finally {
    loading.value = false;
  }
}

// Coming back to the window (e.g. after editing in an IDE) usually means files changed.
onMounted(() => {
  refresh();
  window.addEventListener("focus", refresh);
});

onUnmounted(() => {
  window.removeEventListener("focus", refresh);
});
</script>

<template>
  <section class="repo-view">
    <header class="repo-header">
      <div>
        <h1>{{ repo.name }}</h1>
        <p class="muted path">{{ repo.path }}</p>
      </div>
      <button :disabled="loading" title="Reload changed files" @click="refresh">
        {{ loading ? "Refreshing…" : "Refresh" }}
      </button>
    </header>

    <p v-if="error" class="err">{{ error.message }}</p>

    <template v-if="status">
      <p v-if="changeCount === 0" class="muted">Nothing to commit, working tree clean.</p>
      <FileTable v-else :status="status" />
    </template>
    <p v-else-if="loading" class="muted">Loading…</p>
  </section>
</template>

<style scoped>
.repo-view {
  padding: 20px 28px;
}

.repo-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 16px;
}

h1 {
  margin: 0;
  font-size: 20px;
}

.path {
  margin: 2px 0 0;
  word-break: break-all;
}

.muted {
  color: var(--text-muted);
}
</style>
