<script setup lang="ts">
import { ref, watch } from "vue";
import { useDialogStore } from "../stores/dialogs";
import type { MergeMode } from "../types/merge";

// "Merge X into Y": the three choices of SmartGit's merge dialog.
const dialogs = useDialogStore();

const mode = ref<MergeMode>("mergeCommit");
const message = ref<string | null>(null);
const busy = ref(false);

watch(
  () => dialogs.mergeRequest,
  (request) => {
    if (request !== null) {
      // Always a merge commit by default (your choice), even when a
      // fast-forward would be possible.
      mode.value = "mergeCommit";
      message.value = null;
    }
  },
);

async function onSubmit() {
  const request = dialogs.mergeRequest;
  if (request === null || busy.value) {
    return;
  }
  busy.value = true;
  try {
    message.value = await request.submit(mode.value);
    if (message.value === null) {
      dialogs.closeMerge();
    }
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div v-if="dialogs.mergeRequest" class="backdrop" @mousedown.self="dialogs.closeMerge()">
    <form class="dialog merge-dialog" @submit.prevent="onSubmit" @keydown.esc="dialogs.closeMerge()">
      <h2>Merge "{{ dialogs.mergeRequest.preview.source }}" into "{{ dialogs.mergeRequest.preview.branch }}"</h2>
      <label class="option" :class="{ disabled: !dialogs.mergeRequest.preview.canFastForward }">
        <input v-model="mode" type="radio" value="fastForward" :disabled="!dialogs.mergeRequest.preview.canFastForward" />
        <span>
          <strong>Fast-Forward</strong>: move "{{ dialogs.mergeRequest.preview.branch }}" forward to
          "{{ dialogs.mergeRequest.preview.source }}". No new commit.
          <em v-if="!dialogs.mergeRequest.preview.canFastForward" class="muted">
            Not possible: the branch has commits of its own.
          </em>
        </span>
      </label>
      <label class="option">
        <input v-model="mode" type="radio" value="mergeCommit" />
        <span><strong>Create Merge-Commit</strong>: join both lines of history with a new merge commit.</span>
      </label>
      <label class="option">
        <input v-model="mode" type="radio" value="workingTree" />
        <span>
          <strong>Merge to Working Tree</strong>: bring the changes into the working tree and stop before committing.
          Review them, then commit to create the merge commit.
        </span>
      </label>
      <p v-if="message" class="err">{{ message }}</p>
      <div class="buttons">
        <button type="button" @click="dialogs.closeMerge()">Cancel</button>
        <button type="submit" class="primary" :disabled="busy">Merge</button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.merge-dialog {
  width: 460px;
}

h2 {
  overflow-wrap: anywhere;
}

.option {
  display: flex;
  gap: 8px;
  align-items: flex-start;
  margin: 8px 0;
}

.option input {
  margin-top: 3px;
}

.option.disabled {
  color: var(--text-muted);
}

.muted {
  display: block;
  color: var(--text-muted);
}
</style>
