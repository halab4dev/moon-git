<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { suggestFolderName } from "../api/clone";
import { cancelOperation, onGitProgress } from "../api/operations";
import { toAppError } from "../api/errors";
import { useDialogStore } from "../stores/dialogs";
import { useRepoStore } from "../stores/repos";

// Remembered between clones as a convenience; only this browser view sees it.
const LAST_PARENT_KEY = "moongit.clone.lastParentDir";

const dialogs = useDialogStore();
const store = useRepoStore();

const url = ref("");
const parentDir = ref("");
const folderName = ref("");
// Once the user types a folder name, stop replacing it with suggestions.
const folderEdited = ref(false);

const running = ref(false);
const cancelling = ref(false);
const opId = ref<string | null>(null);
const phase = ref("");
const percent = ref<number | null>(null);
const lastLine = ref("");
const error = ref<string | null>(null);

const urlInput = ref<HTMLInputElement | null>(null);

const groupLabel = computed(() => {
  const groupId = dialogs.cloneRequest?.groupId ?? null;
  return store.groupOptions.find((option) => option.id === groupId)?.label ?? null;
});

const targetPath = computed(() => {
  const name = folderName.value.trim();
  return parentDir.value && name ? `${parentDir.value.replace(/[\\/]+$/, "")}/${name}` : "";
});

const canStart = computed(
  () => !running.value && url.value.trim() !== "" && parentDir.value.trim() !== "" && folderName.value.trim() !== "",
);

watch(
  () => dialogs.cloneRequest,
  async (request) => {
    if (request === null) {
      return;
    }
    url.value = "";
    folderName.value = "";
    folderEdited.value = false;
    parentDir.value = readLastParent();
    resetProgress();
    error.value = null;
    await nextTick();
    urlInput.value?.focus();
  },
);

watch(url, async (value) => {
  if (folderEdited.value) {
    return;
  }
  const suggestion = await suggestFolderName(value);
  // The URL may have changed again while we waited; only apply the latest answer.
  if (url.value === value && !folderEdited.value) {
    folderName.value = suggestion;
  }
});

async function browse() {
  const picked = await open({
    directory: true,
    multiple: false,
    title: "Choose Parent Folder",
    defaultPath: parentDir.value || undefined,
  });
  if (typeof picked === "string") {
    parentDir.value = picked;
  }
}

async function start() {
  const request = dialogs.cloneRequest;
  if (request === null || !canStart.value) {
    return;
  }

  const id = crypto.randomUUID();
  opId.value = id;
  running.value = true;
  cancelling.value = false;
  error.value = null;
  resetProgress();
  lastLine.value = "Starting…";
  rememberParent(parentDir.value);

  // Listen before starting so no early progress line is missed.
  const unlisten = await onGitProgress((progress) => {
    if (progress.opId !== id) {
      return;
    }
    lastLine.value = progress.line;
    if (progress.phase !== null && progress.percent !== null) {
      phase.value = progress.phase;
      percent.value = progress.percent;
    }
  });

  try {
    await store.cloneRepo(id, url.value.trim(), parentDir.value.trim(), folderName.value.trim(), request.groupId);
    dialogs.closeClone();
  } catch (err) {
    const appError = toAppError(err);
    // Cancelling was the user's choice: go back to the form without an error.
    error.value = appError.kind === "cancelled" ? null : appError.message;
  } finally {
    unlisten();
    running.value = false;
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
    error.value = toAppError(err).message;
    cancelling.value = false;
  }
}

// The dialog can't be closed while git is running; use Cancel Clone first.
function close() {
  if (!running.value) {
    dialogs.closeClone();
  }
}

function resetProgress() {
  phase.value = "";
  percent.value = null;
  lastLine.value = "";
}

function readLastParent(): string {
  try {
    return localStorage.getItem(LAST_PARENT_KEY) ?? "";
  } catch {
    return "";
  }
}

function rememberParent(dir: string) {
  try {
    localStorage.setItem(LAST_PARENT_KEY, dir);
  } catch {
    // Storage may be unavailable; remembering the folder is only a convenience.
  }
}
</script>

<template>
  <div v-if="dialogs.cloneRequest" class="backdrop" @mousedown.self="close">
    <form class="dialog clone" @submit.prevent="start" @keydown.esc="close">
      <h2>Clone Repository</h2>

      <label>
        <span>Repository URL</span>
        <input
          ref="urlInput"
          v-model="url"
          :disabled="running"
          placeholder="https://github.com/user/repo.git  or  git@github.com:user/repo.git"
          spellcheck="false"
        />
      </label>

      <label>
        <span>Parent folder</span>
        <div class="with-button">
          <input v-model="parentDir" :disabled="running" placeholder="Choose a folder…" spellcheck="false" />
          <button type="button" :disabled="running" @click="browse">Browse…</button>
        </div>
      </label>

      <label>
        <span>Folder name</span>
        <input v-model="folderName" :disabled="running" spellcheck="false" @input="folderEdited = true" />
      </label>

      <p v-if="targetPath" class="muted">
        Clone into <code>{{ targetPath }}</code>
        <template v-if="groupLabel"> · group <strong>{{ groupLabel }}</strong></template>
      </p>

      <div v-if="running" class="progress">
        <!-- Without a value, <progress> shows an "in progress" animation. -->
        <progress max="100" :value="percent ?? undefined"></progress>
        <p class="muted line">{{ phase ? `${phase} ${percent}%` : lastLine }}</p>
      </div>

      <p v-if="error" class="err message">{{ error }}</p>

      <div class="buttons">
        <button v-if="running" type="button" :disabled="cancelling" @click="cancel">
          {{ cancelling ? "Cancelling…" : "Cancel Clone" }}
        </button>
        <template v-else>
          <button type="button" @click="close">Close</button>
          <button type="submit" class="primary" :disabled="!canStart">Clone</button>
        </template>
      </div>
    </form>
  </div>
</template>

<style scoped>
.clone {
  width: 520px;
}

label {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

label > span {
  font-weight: 600;
}

.with-button {
  display: flex;
  gap: 8px;
}

.with-button input {
  flex: 1;
}

.muted {
  color: var(--text-muted);
  word-break: break-all;
}

.progress progress {
  width: 100%;
}

.line {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* git error messages can span several lines. */
.message {
  white-space: pre-wrap;
  word-break: break-word;
}
</style>
