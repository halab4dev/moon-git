<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import { useDialogStore } from "../stores/dialogs";

const dialogs = useDialogStore();

const value = ref("");
const message = ref<string | null>(null);
const busy = ref(false);
// Template ref: filled by Vue with the <input> element once it is rendered.
const input = ref<HTMLInputElement | null>(null);

// Reset the form each time a new request opens the dialog.
watch(
  () => dialogs.nameRequest,
  async (request) => {
    if (request === null) {
      return;
    }
    value.value = request.initial;
    message.value = null;
    await nextTick(); // wait for the <input> to exist
    input.value?.focus();
    input.value?.select();
  },
);

async function onSubmit() {
  const request = dialogs.nameRequest;
  if (request === null || busy.value) {
    return;
  }
  busy.value = true;
  try {
    message.value = await request.submit(value.value);
    if (message.value === null) {
      dialogs.closeName();
    }
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div v-if="dialogs.nameRequest" class="backdrop" @mousedown.self="dialogs.closeName()">
    <form class="dialog" @submit.prevent="onSubmit" @keydown.esc="dialogs.closeName()">
      <h2>{{ dialogs.nameRequest.title }}</h2>
      <input ref="input" v-model="value" placeholder="Name" />
      <p v-if="message" class="err">{{ message }}</p>
      <div class="buttons">
        <button type="button" @click="dialogs.closeName()">Cancel</button>
        <button type="submit" class="primary" :disabled="busy || !value.trim()">
          {{ dialogs.nameRequest.confirmLabel }}
        </button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.dialog {
  width: 360px;
}

.dialog input {
  width: 100%;
}
</style>
