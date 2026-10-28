<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import { useDialogStore } from "../stores/dialogs";

// "Sign in to github.com" (step 13b): opened by withSignIn() (signIn.ts) when
// a server wants a login. The token is kept in this component only while the
// dialog is open, and goes to Rust (then the keyring) when Sign In is clicked.
const dialogs = useDialogStore();

const username = ref("");
const token = ref("");
const message = ref<string | null>(null);
const busy = ref(false);
const usernameInput = ref<HTMLInputElement | null>(null);
const tokenInput = ref<HTMLInputElement | null>(null);

watch(
  () => dialogs.signInRequest,
  async (request) => {
    token.value = "";
    if (request === null) {
      return;
    }
    username.value = request.username;
    message.value = null;
    await nextTick();
    (request.username ? tokenInput : usernameInput).value?.focus();
  },
);

async function onSubmit() {
  const request = dialogs.signInRequest;
  if (request === null || busy.value) {
    return;
  }
  busy.value = true;
  try {
    message.value = await request.submit(username.value, token.value);
    if (message.value === null) {
      dialogs.closeSignIn();
    } else {
      // Refused: type the token again.
      token.value = "";
      tokenInput.value?.focus();
    }
  } finally {
    busy.value = false;
  }
}

function onCancel() {
  if (busy.value) {
    return;
  }
  dialogs.signInRequest?.cancel();
  dialogs.closeSignIn();
}
</script>

<template>
  <div v-if="dialogs.signInRequest" class="backdrop" @mousedown.self="onCancel">
    <form class="dialog sign-in-dialog" @submit.prevent="onSubmit" @keydown.esc="onCancel">
      <h2>Sign in to {{ dialogs.signInRequest.server }}</h2>
      <p class="muted">{{ dialogs.signInRequest.reason }}</p>
      <label class="field">
        <span>User name</span>
        <input ref="usernameInput" v-model="username" autocomplete="off" spellcheck="false" :disabled="busy" />
      </label>
      <label class="field">
        <span>Password or token</span>
        <input ref="tokenInput" v-model="token" type="password" autocomplete="off" :disabled="busy" />
      </label>
      <p class="muted hint">
        GitHub, GitLab and Bitbucket need a <strong>personal access token</strong> here, not your account password.
        It is saved in the system keychain for {{ dialogs.signInRequest.server }} and used for its other repositories
        too.
      </p>
      <p v-if="message" class="err">{{ message }}</p>
      <div class="buttons">
        <button type="button" :disabled="busy" @click="onCancel">Cancel</button>
        <button type="submit" class="primary" :disabled="busy || !username.trim() || !token.trim()">
          {{ busy ? "Signing in…" : "Sign In" }}
        </button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.sign-in-dialog {
  width: 440px;
}

.field {
  display: flex;
  align-items: center;
  gap: 8px;
}

.field span {
  width: 120px;
  flex-shrink: 0;
}

.field input {
  flex: 1;
  min-width: 0;
}

.hint {
  font-size: 12px;
}

.muted {
  color: var(--text-muted);
}
</style>
