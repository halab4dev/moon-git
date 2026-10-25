<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import { useDialogStore } from "../stores/dialogs";

// "Add Remote": a name ("origin", "upstream"…) and the server URL.
// "Edit Remote": the same dialog, only the URL can be changed.
const dialogs = useDialogStore();

const name = ref("");
const url = ref("");
const message = ref<string | null>(null);
const busy = ref(false);
const nameInput = ref<HTMLInputElement | null>(null);
const urlInput = ref<HTMLInputElement | null>(null);

watch(
  () => dialogs.remoteRequest,
  async (request) => {
    if (request === null) {
      return;
    }
    name.value = request.existing?.name ?? request.initialName;
    url.value = request.existing?.url ?? "";
    message.value = null;
    await nextTick();
    // With a name given, the URL is what's left to type.
    (name.value ? urlInput : nameInput).value?.focus();
    if (request.existing) {
      urlInput.value?.select();
    }
  },
);

async function onSubmit() {
  const request = dialogs.remoteRequest;
  if (request === null || busy.value) {
    return;
  }
  busy.value = true;
  try {
    message.value = await request.submit(name.value, url.value);
    if (message.value === null) {
      dialogs.closeRemote();
    }
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div v-if="dialogs.remoteRequest" class="backdrop" @mousedown.self="dialogs.closeRemote()">
    <form class="dialog remote-dialog" @submit.prevent="onSubmit" @keydown.esc="dialogs.closeRemote()">
      <h2>{{ dialogs.remoteRequest.existing ? `Edit Remote ${dialogs.remoteRequest.existing.name}` : "Add Remote" }}</h2>
      <label class="field">
        <span>Name</span>
        <input
          ref="nameInput"
          v-model="name"
          placeholder="origin"
          spellcheck="false"
          :readonly="dialogs.remoteRequest.existing !== null"
        />
      </label>
      <label class="field">
        <span>URL</span>
        <input
          ref="urlInput"
          v-model="url"
          placeholder="https://github.com/user/repo.git or git@github.com:user/repo.git"
          spellcheck="false"
        />
      </label>
      <p class="muted">
        {{
          dialogs.remoteRequest.existing
            ? "The branches already fetched stay; the next Fetch uses the new URL."
            : "Nothing is downloaded yet: its branches appear after the next Fetch."
        }}
      </p>
      <p v-if="message" class="err">{{ message }}</p>
      <div class="buttons">
        <button type="button" @click="dialogs.closeRemote()">Cancel</button>
        <button type="submit" class="primary" :disabled="busy || !name.trim() || !url.trim()">
          {{ dialogs.remoteRequest.existing ? "Save" : "Add" }}
        </button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.remote-dialog {
  width: 480px;
}

.field {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 8px 0;
}

.field span {
  width: 44px;
  flex-shrink: 0;
}

.field input {
  flex: 1;
  min-width: 0;
}

.field input[readonly] {
  color: var(--text-muted);
}

.muted {
  color: var(--text-muted);
}
</style>
