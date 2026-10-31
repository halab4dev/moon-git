<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { getIdentity, setIdentity } from "../api/identity";
import { toAppError } from "../api/errors";
import { useDialogStore } from "../stores/dialogs";
import type { Identity } from "../types/identity";

// "Commit Identity" of one repository (step 14): the name and email put on
// its commits. Either the computer-wide one (~/.gitconfig), or its own
// (written to the repository's .git/config).
const dialogs = useDialogStore();

const identity = ref<Identity | null>(null);
const own = ref(false);
const name = ref("");
const email = ref("");
const message = ref<string | null>(null);
const busy = ref(false);
const nameInput = ref<HTMLInputElement | null>(null);

watch(
  () => dialogs.identityRequest,
  async (request) => {
    identity.value = null;
    message.value = null;
    if (request === null) {
      return;
    }
    try {
      const loaded = await getIdentity(request.repoId);
      identity.value = loaded;
      own.value = loaded.name !== null || loaded.email !== null;
      // Start from what applies now, so only the different part needs typing.
      name.value = loaded.name ?? loaded.inheritedName ?? "";
      email.value = loaded.email ?? loaded.inheritedEmail ?? "";
    } catch (err) {
      message.value = toAppError(err).message;
    }
  },
);

// Choosing "Only for this repository" puts the cursor in Name.
watch(own, async (value) => {
  if (value) {
    await nextTick();
    nameInput.value?.focus();
  }
});

const inheritedText = computed(() => {
  const id = identity.value;
  if (id === null) {
    return "";
  }
  if (id.inheritedName === null && id.inheritedEmail === null) {
    return "not set on this computer: commits will fail";
  }
  return `${id.inheritedName ?? "(no name)"} <${id.inheritedEmail ?? "no email"}>`;
});

async function onSubmit() {
  const request = dialogs.identityRequest;
  if (request === null || busy.value || identity.value === null) {
    return;
  }
  busy.value = true;
  message.value = null;
  try {
    await setIdentity(request.repoId, own.value ? { name: name.value, email: email.value } : null);
    request.saved?.();
    dialogs.closeIdentity();
  } catch (err) {
    message.value = toAppError(err).message;
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div v-if="dialogs.identityRequest" class="backdrop" @mousedown.self="dialogs.closeIdentity()">
    <form class="dialog identity-dialog" @submit.prevent="onSubmit" @keydown.esc="dialogs.closeIdentity()">
      <h2>Commit Identity · {{ dialogs.identityRequest.repoName }}</h2>
      <p class="muted">The name and email written into every commit made in this repository.</p>

      <template v-if="identity">
        <label class="option">
          <input v-model="own" type="radio" :value="false" />
          <span>
            <strong>Same as the rest of this computer</strong>
            <span class="muted detail" :class="{ warn: identity.inheritedEmail === null }">{{ inheritedText }}</span>
          </span>
        </label>
        <label class="option">
          <input v-model="own" type="radio" :value="true" />
          <span><strong>Only for this repository</strong> (e.g. a work email)</span>
        </label>
        <div class="fields" :class="{ disabled: !own }">
          <label class="field">
            <span>Name</span>
            <input ref="nameInput" v-model="name" :disabled="!own" placeholder="Your Name" spellcheck="false" />
          </label>
          <label class="field">
            <span>Email</span>
            <input v-model="email" :disabled="!own" placeholder="you@example.com" spellcheck="false" />
          </label>
        </div>
      </template>
      <p v-else-if="!message" class="muted">Loading…</p>

      <p v-if="message" class="err">{{ message }}</p>
      <div class="buttons">
        <button type="button" @click="dialogs.closeIdentity()">Cancel</button>
        <button
          type="submit"
          class="primary"
          :disabled="busy || identity === null || (own && (!name.trim() || !email.trim()))"
        >
          Save
        </button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.identity-dialog {
  width: 480px;
}

.option {
  display: flex;
  gap: 8px;
  align-items: flex-start;
}

.option input {
  margin-top: 3px;
}

.detail {
  display: block;
}

.fields {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-left: 24px;
}

.fields.disabled {
  opacity: 0.55;
}

.field {
  display: flex;
  align-items: center;
  gap: 8px;
}

.field span {
  width: 48px;
  flex-shrink: 0;
}

.field input {
  flex: 1;
  min-width: 0;
}

.muted {
  color: var(--text-muted);
}

.warn {
  color: var(--warning);
}
</style>
