<script setup lang="ts">
import { onMounted, ref, shallowRef, watch } from "vue";
import CommitList from "../components/CommitList.vue";
import { getLog } from "../api/log";
import { toAppError } from "../api/errors";
import type { AppError } from "../types/error";
import type { CommitSummary } from "../types/log";

// "History" tab: all branches, remote branches and tags, newest first,
// loaded page by page while scrolling.
const props = defineProps<{
  repoId: string;
  /** Changes whenever the status was reloaded: new commits may exist. */
  version: number;
}>();

const PAGE_SIZE = 500;
const MAX_PAGE = 2000; // limit of the Rust side

// shallowRef: Vue only watches the array as a whole, not every field of every
// commit. With tens of thousands of commits, deep watching would be slow.
// Consequence: always replace the array, never push into it.
const commits = shallowRef<CommitSummary[]>([]);
const hasMore = ref(false);
const loading = ref(false);
const error = ref<AppError | null>(null);
const selected = ref<string | null>(null);

// Bumped by every reload; an answer for an older generation is dropped
// (e.g. a page requested just before the list was reloaded).
let generation = 0;

async function reload() {
  const current = ++generation;
  loading.value = true;
  // Keep about as many commits as are loaded now, so the scroll position survives.
  const count = Math.min(MAX_PAGE, Math.max(PAGE_SIZE, commits.value.length));
  try {
    const page = await getLog(props.repoId, 0, count);
    if (current !== generation) {
      return;
    }
    commits.value = page.commits;
    hasMore.value = page.hasMore;
    error.value = null;
  } catch (err) {
    if (current === generation) {
      error.value = toAppError(err);
    }
  } finally {
    if (current === generation) {
      loading.value = false;
    }
  }
}

async function loadMore() {
  if (loading.value || !hasMore.value) {
    return;
  }
  const current = generation;
  loading.value = true;
  try {
    const page = await getLog(props.repoId, commits.value.length, PAGE_SIZE);
    if (current !== generation) {
      return;
    }
    commits.value = commits.value.concat(page.commits);
    hasMore.value = page.hasMore;
  } catch (err) {
    if (current === generation) {
      error.value = toAppError(err);
    }
  } finally {
    if (current === generation) {
      loading.value = false;
    }
  }
}

onMounted(reload);
watch(() => props.version, reload);
</script>

<template>
  <div class="history">
    <p v-if="error" class="err message">{{ error.message }}</p>
    <CommitList
      v-model:selected="selected"
      :commits="commits"
      :has-more="hasMore"
      :loading="loading"
      @load-more="loadMore"
    />
    <p class="muted footer">
      {{ commits.length }} commit(s) loaded{{ hasMore ? ", more below" : "" }}{{ loading ? " · loading…" : "" }}
    </p>
  </div>
</template>

<style scoped>
.history {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.footer {
  margin: 4px 0 0;
  font-size: 12px;
}

.muted {
  color: var(--text-muted);
}

.message {
  white-space: pre-wrap;
}
</style>
