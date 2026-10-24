<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { ask } from "@tauri-apps/plugin-dialog";
import { useDialogStore } from "../stores/dialogs";
import type { ResetMode } from "../types/reset";

// "Reset main to 1234abcd": Soft / Mixed / Hard, like SmartGit.
// Hard throws uncommitted changes away, so it asks once more on submit.
const dialogs = useDialogStore();

const mode = ref<ResetMode>("mixed");
const message = ref<string | null>(null);
const busy = ref(false);

const preview = computed(() => dialogs.resetRequest?.preview ?? null);

watch(
  () => dialogs.resetRequest,
  (request) => {
    if (request !== null) {
      // git's default, and safe: nothing is lost.
      mode.value = "mixed";
      message.value = null;
    }
  },
);

async function onSubmit() {
  const request = dialogs.resetRequest;
  if (request === null || busy.value) {
    return;
  }
  if (mode.value === "hard") {
    const files = request.preview.changedFiles;
    const lost =
      files > 0
        ? `The uncommitted changes in ${files} file(s) are lost for good.`
        : "There are no uncommitted changes now, but the files are reset to that commit.";
    const confirmed = await ask(`Hard reset "${request.preview.branch}" to ${request.preview.target}?\n\n${lost}`, {
      title: "Hard Reset",
      kind: "warning",
      okLabel: "Reset and Discard",
      cancelLabel: "Cancel",
    });
    if (!confirmed) {
      return;
    }
  }
  busy.value = true;
  try {
    message.value = await request.submit(mode.value);
    if (message.value === null) {
      dialogs.closeReset();
    }
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div v-if="preview" class="backdrop" @mousedown.self="dialogs.closeReset()">
    <form class="dialog reset-dialog" @submit.prevent="onSubmit" @keydown.esc="dialogs.closeReset()">
      <h2>Reset "{{ preview.branch }}" to {{ preview.target }}</h2>
      <p v-if="preview.isHead" class="muted">This is the current commit: the branch stays, only the files can change.</p>
      <p v-else>
        "{{ preview.branch }}" moves to {{ preview.target }}.
        <template v-if="preview.leftBehind > 0">
          {{ preview.leftBehind }} commit(s) are left behind (no longer on the branch).
        </template>
      </p>
      <p v-if="preview.pushed > 0" class="warn">
        {{ preview.pushed }} of them are already pushed. Afterwards, pushing needs --force (MoonGit never forces).
      </p>

      <label class="option">
        <input v-model="mode" type="radio" value="soft" />
        <span><strong>Soft</strong>: only move the branch. The changes of the left-behind commits stay <em>staged</em>, ready to be committed again.</span>
      </label>
      <label class="option">
        <input v-model="mode" type="radio" value="mixed" />
        <span><strong>Mixed</strong>: move the branch and unstage. Those changes stay in the files, <em>unstaged</em>.</span>
      </label>
      <label class="option">
        <input v-model="mode" type="radio" value="hard" />
        <span>
          <strong>Hard</strong>: move the branch and reset all files to that commit.
          <span class="danger">
            {{ preview.changedFiles > 0 ? `Your uncommitted changes in ${preview.changedFiles} file(s) are lost.` : "Uncommitted changes would be lost." }}
          </span>
          Untracked files stay.
        </span>
      </label>

      <p v-if="message" class="err">{{ message }}</p>
      <div class="buttons">
        <button type="button" @click="dialogs.closeReset()">Cancel</button>
        <button type="submit" class="primary" :class="{ destructive: mode === 'hard' }" :disabled="busy">Reset</button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.reset-dialog {
  width: 480px;
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

.muted {
  color: var(--text-muted);
}

.warn {
  color: var(--warning);
}

.danger {
  color: var(--danger);
}

button.destructive {
  border-color: var(--danger);
  background: var(--danger);
}
</style>
