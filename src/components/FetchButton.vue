<script setup lang="ts">
import { ref } from "vue";
import { fetchRepo } from "../api/fetch";
import { cancelOperation, onGitProgress } from "../api/operations";
import { toAppError } from "../api/errors";
import type { AppError } from "../types/error";

// "Fetch" button that turns into a progress line + Cancel while git runs.
// It does not use the parent's busy flag: fetch only updates remote-tracking
// branches, so staging and committing can go on meanwhile.
const props = defineProps<{
  repoId: string;
}>();

const emit = defineEmits<{
  /** Fetch finished (or was cancelled): time to reload the status. */
  done: [];
  failed: [error: AppError];
}>();

const opId = ref<string | null>(null);
const progress = ref("");
const cancelling = ref(false);

async function start() {
  if (opId.value !== null) {
    return;
  }
  const id = crypto.randomUUID();
  opId.value = id;
  cancelling.value = false;
  progress.value = "Fetching…";

  // Listen before starting so no early progress line is missed.
  const unlisten = await onGitProgress((line) => {
    if (line.opId === id) {
      progress.value = line.phase !== null ? `${line.phase} ${line.percent}%` : line.line;
    }
  });

  try {
    await fetchRepo(id, props.repoId);
    emit("done");
  } catch (err) {
    const error = toAppError(err);
    // Cancelling was the user's choice, not an error.
    if (error.kind === "cancelled") {
      emit("done");
    } else {
      emit("failed", error);
    }
  } finally {
    unlisten();
    opId.value = null;
  }
}

async function cancel() {
  if (opId.value === null) {
    return;
  }
  cancelling.value = true;
  try {
    await cancelOperation(opId.value);
  } catch (err) {
    cancelling.value = false;
    emit("failed", toAppError(err));
  }
}
</script>

<template>
  <span class="fetch">
    <template v-if="opId === null">
      <button title="Download new commits from all remotes (files stay unchanged)" @click="start">Fetch</button>
    </template>
    <template v-else>
      <span class="progress muted" :title="progress">{{ progress }}</span>
      <button :disabled="cancelling" @click="cancel">{{ cancelling ? "Cancelling…" : "Cancel" }}</button>
    </template>
  </span>
</template>

<style scoped>
.fetch {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.progress {
  max-width: 280px;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.muted {
  color: var(--text-muted);
}
</style>
