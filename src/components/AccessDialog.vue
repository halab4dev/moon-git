<script setup lang="ts">
import { ref, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { homeDir, join } from "@tauri-apps/api/path";
import { getRemoteAccess, setRemoteAccess } from "../api/access";
import { toAppError } from "../api/errors";
import { useDialogStore } from "../stores/dialogs";
import type { RemoteAccess } from "../types/access";

// Repository → Login (SSH & HTTPS)… (step 15a). A repository can use both:
// e.g. origin over SSH and upstream over HTTPS, so both parts are always shown. Both choices are saved in the
// repository's .git/config, so git in a terminal behaves the same:
//   - SSH: this computer's settings (~/.ssh/config, ssh-agent…) or one key;
//   - HTTPS: MoonGit's accounts (Sign in dialog) or this computer's
//     credential helpers (e.g. the macOS keychain via osxkeychain).
const dialogs = useDialogStore();

const access = ref<RemoteAccess | null>(null);
const useKey = ref(false);
const keyPath = ref("");
const systemCredentials = ref(false);
const message = ref<string | null>(null);
const busy = ref(false);

watch(
  () => dialogs.accessRequest,
  async (request) => {
    access.value = null;
    message.value = null;
    if (request === null) {
      return;
    }
    try {
      const loaded = await getRemoteAccess(request.repoId);
      access.value = loaded;
      useKey.value = loaded.sshKey !== null;
      keyPath.value = loaded.sshKey ?? "";
      systemCredentials.value = loaded.systemCredentials;
    } catch (err) {
      message.value = toAppError(err).message;
    }
  },
);

async function chooseKey() {
  let start: string | undefined;
  try {
    start = await join(await homeDir(), ".ssh");
  } catch {
    start = undefined;
  }
  const picked = await open({ title: "Choose the private SSH key", defaultPath: start, multiple: false, directory: false });
  if (typeof picked === "string") {
    keyPath.value = picked;
    useKey.value = true;
  }
}

// "Used by: origin, upstream", or why the part doesn't matter right now.
function usedBy(remotes: string[], kind: string): string {
  return remotes.length > 0
    ? `Used by: ${remotes.join(", ")}`
    : `No remote of this repository uses ${kind} now (applies if one is added).`;
}

async function onSubmit() {
  const request = dialogs.accessRequest;
  if (request === null || busy.value || access.value === null) {
    return;
  }
  busy.value = true;
  message.value = null;
  try {
    await setRemoteAccess(request.repoId, useKey.value ? keyPath.value : null, systemCredentials.value);
    dialogs.closeAccess();
  } catch (err) {
    message.value = toAppError(err).message;
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div v-if="dialogs.accessRequest" class="backdrop" @mousedown.self="dialogs.closeAccess()">
    <form class="dialog access-dialog" @submit.prevent="onSubmit" @keydown.esc="dialogs.closeAccess()">
      <h2>Login · {{ dialogs.accessRequest.repoName }}</h2>
      <p class="muted small">How this repository logs in to its servers, for each kind of remote URL.</p>

      <template v-if="access">
        <h3>SSH <span class="muted">(remote URLs like git@github.com:…)</span></h3>
        <p class="used small">{{ usedBy(access.sshRemotes, "SSH") }}</p>
        <label class="option">
          <input v-model="useKey" type="radio" :value="false" />
          <span>
            <strong>Use this computer's SSH settings</strong>
            <span class="muted detail">~/.ssh/config, the keys in ssh-agent, ~/.ssh/id_ed25519…</span>
          </span>
        </label>
        <label class="option">
          <input v-model="useKey" type="radio" :value="true" />
          <span><strong>Use this key</strong> <span class="muted">(only this one is offered to the server)</span></span>
        </label>
        <div class="key" :class="{ disabled: !useKey }">
          <input v-model="keyPath" :disabled="!useKey" placeholder="~/.ssh/id_work (the private key, not .pub)" spellcheck="false" />
          <button type="button" @click="chooseKey">Choose…</button>
        </div>
        <p v-if="access.customSshCommand" class="warn small">
          This repository has its own SSH command, set by hand: <code>{{ access.customSshCommand }}</code>. Saving
          replaces it.
        </p>

        <h3>HTTPS <span class="muted">(remote URLs like https://github.com/…)</span></h3>
        <p class="used small">{{ usedBy(access.httpsRemotes, "HTTPS") }}</p>
        <label class="option">
          <input v-model="systemCredentials" type="radio" :value="false" />
          <span>
            <strong>MoonGit's accounts</strong>
            <span class="muted detail">Asks you to sign in when needed; saved in MoonGit → Accounts….</span>
          </span>
        </label>
        <label class="option">
          <input v-model="systemCredentials" type="radio" :value="true" />
          <span>
            <strong>Use this computer's credentials</strong>
            <span class="muted detail">git's own credential helpers (e.g. the macOS keychain), as in a terminal.</span>
          </span>
        </label>
        <p class="muted small">Saved in this repository's <code>.git/config</code>: git in a terminal does the same.</p>
      </template>
      <p v-else-if="!message" class="muted">Loading…</p>

      <p v-if="message" class="err">{{ message }}</p>
      <div class="buttons">
        <button type="button" @click="dialogs.closeAccess()">Cancel</button>
        <button type="submit" class="primary" :disabled="busy || access === null || (useKey && !keyPath.trim())">
          Save
        </button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.access-dialog {
  width: 520px;
}

h3 {
  margin: 6px 0 0;
  font-size: 13px;
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

.key {
  display: flex;
  gap: 6px;
  margin-left: 24px;
}

.key input {
  flex: 1;
  min-width: 0;
}

.key.disabled input {
  opacity: 0.55;
}

.used {
  margin: 0;
  color: var(--accent);
}

.small {
  font-size: 12px;
  overflow-wrap: anywhere;
}

.muted {
  color: var(--text-muted);
  font-weight: 400;
}

.warn {
  color: var(--warning);
}
</style>
