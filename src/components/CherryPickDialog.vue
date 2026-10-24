<script setup lang="ts">
import { ref, watch } from "vue";
import { useDialogStore } from "../stores/dialogs";
import type { CherryPickMode } from "../types/cherryPick";

// "Cherry-Pick 1234abcd": commit it again, or only bring its changes in.
const dialogs = useDialogStore();

const mode = ref<CherryPickMode>("commit");
const message = ref<string | null>(null);
const busy = ref(false);

watch(
  () => dialogs.cherryPickRequest,
  (request) => {
    if (request !== null) {
      mode.value = "commit";
      message.value = null;
    }
  },
);

async function onSubmit() {
  const request = dialogs.cherryPickRequest;
  if (request === null || busy.value) {
    return;
  }
  busy.value = true;
  try {
    message.value = await request.submit(mode.value);
    if (message.value === null) {
      dialogs.closeCherryPick();
    }
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div v-if="dialogs.cherryPickRequest" class="backdrop" @mousedown.self="dialogs.closeCherryPick()">
    <form class="dialog pick-dialog" @submit.prevent="onSubmit" @keydown.esc="dialogs.closeCherryPick()">
      <h2>Cherry-Pick {{ dialogs.cherryPickRequest.commitId.slice(0, 8) }}</h2>
      <p class="subject">"{{ dialogs.cherryPickRequest.subject }}"</p>
      <p v-if="dialogs.cherryPickRequest.isMerge" class="muted">
        This is a merge commit: its changes are taken relative to its first parent (as shown in History).
      </p>
      <label class="option">
        <input v-model="mode" type="radio" value="commit" />
        <span><strong>Commit</strong>: a new commit on the current branch, with the same changes, message and author.</span>
      </label>
      <label class="option">
        <input v-model="mode" type="radio" value="workingTree" />
        <span>
          <strong>Apply to Working Tree</strong>: only bring the changes in, staged. Review them, then commit them
          yourself (or together with other changes).
        </span>
      </label>
      <p v-if="message" class="err">{{ message }}</p>
      <div class="buttons">
        <button type="button" @click="dialogs.closeCherryPick()">Cancel</button>
        <button type="submit" class="primary" :disabled="busy">Cherry-Pick</button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.pick-dialog {
  width: 460px;
}

.subject {
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

.muted {
  color: var(--text-muted);
}
</style>
