<script setup lang="ts">
import { ref, watch } from "vue";
import { message as showMessage, open } from "@tauri-apps/plugin-dialog";
import { homeDir, join } from "@tauri-apps/api/path";
import { toAppError } from "../api/errors";
import { useDialogStore } from "../stores/dialogs";
import { useRepoStore } from "../stores/repos";
import { useApplyTarget } from "./applyTarget";

// Group → Login (SSH & HTTPS)… (step 15b), also for the repositories picked
// in the sidebar (step 35c): the same choices as a
// repository's Login dialog (15a), written into the .git/config of the
// repositories of the group and its subgroups, with the two ways of step 14b:
//   - Only repositories without their own setting (SSH and HTTPS each on
//     their own: a repository with its own key keeps it, but can still get
//     the HTTPS choice);
//   - Write for all repositories.
const dialogs = useDialogStore();
const store = useRepoStore();

const useKey = ref(false);
const keyPath = ref("");
const systemCredentials = ref(false);
const all = ref(false);
const message = ref<string | null>(null);
const busy = ref(false);

const { group, repoIds, label, repoCount, where, ready } = useApplyTarget(() => dialogs.groupAccessRequest);

watch(
  () => dialogs.groupAccessRequest,
  (request) => {
    message.value = null;
    all.value = false;
    if (request === null || !ready.value) {
      return;
    }
    // What was written for this group last time (or the defaults).
    const last = group.value?.access;
    useKey.value = (last?.sshKey ?? null) !== null;
    keyPath.value = last?.sshKey ?? "";
    systemCredentials.value = last?.systemCredentials ?? false;
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

async function onSubmit() {
  if (!ready.value || busy.value) {
    return;
  }
  const title = `Login · ${label.value}`;
  const key = useKey.value ? keyPath.value : null;
  busy.value = true;
  message.value = null;
  try {
    const result =
      group.value !== null
        ? await store.setGroupAccess(group.value.id, key, systemCredentials.value, all.value)
        : await store.setReposAccess(repoIds.value ?? [], key, systemCredentials.value, all.value);
    dialogs.closeGroupAccess();
    const lines = [
      `Changed in ${result.changed} of ${result.total} ${result.total === 1 ? "repository" : "repositories"}.`,
    ];
    const untouched = result.total - result.changed - result.failed.length;
    if (untouched > 0) {
      lines.push(`${untouched} kept ${untouched === 1 ? "its" : "their"} own settings (or already had these).`);
    }
    if (result.failed.length > 0) {
      lines.push("", "Could not be changed:", ...result.failed);
    }
    void showMessage(lines.join("\n"), {
      title,
      kind: result.failed.length > 0 ? "warning" : "info",
    });
  } catch (err) {
    message.value = toAppError(err).message;
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div v-if="ready" class="backdrop" @mousedown.self="dialogs.closeGroupAccess()">
    <form class="dialog access-dialog" @submit.prevent="onSubmit" @keydown.esc="dialogs.closeGroupAccess()">
      <h2>Login · {{ label }}</h2>
      <p class="muted small">
        Written into the {{ repoCount }} {{ repoCount === 1 ? "repository" : "repositories" }} {{ where }} (their
        <code>.git/config</code>), so git in a terminal does the same.
      </p>

      <h3>SSH <span class="muted">(remote URLs like git@github.com:…)</span></h3>
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
        <input
          v-model="keyPath"
          :disabled="!useKey"
          placeholder="~/.ssh/id_work (the private key, not .pub)"
          spellcheck="false"
        />
        <button type="button" @click="chooseKey">Choose…</button>
      </div>

      <h3>HTTPS <span class="muted">(remote URLs like https://github.com/…)</span></h3>
      <label class="option">
        <input v-model="systemCredentials" type="radio" :value="false" />
        <span><strong>MoonGit's accounts</strong> <span class="muted">(Sign in dialog, MoonGit → Accounts…)</span></span>
      </label>
      <label class="option">
        <input v-model="systemCredentials" type="radio" :value="true" />
        <span>
          <strong>Use this computer's credentials</strong>
          <span class="muted">(git's own helpers, e.g. the macOS keychain)</span>
        </span>
      </label>

      <h3>Write into</h3>
      <label class="option">
        <input v-model="all" type="radio" :value="false" />
        <span>
          <strong>Only repositories without their own setting</strong>
          <span class="muted detail">
            SSH and HTTPS on their own: a repository with its own key keeps it, and still gets the HTTPS choice if it
            has none.
          </span>
        </span>
      </label>
      <label class="option">
        <input v-model="all" type="radio" :value="true" />
        <span>
          <strong>Write for all repositories</strong>
          <span class="muted detail">Also replaces what repositories already have ("this computer's" removes it).</span>
        </span>
      </label>

      <p v-if="message" class="err">{{ message }}</p>
      <div class="buttons">
        <button type="button" @click="dialogs.closeGroupAccess()">Cancel</button>
        <button type="submit" class="primary" :disabled="busy || repoCount === 0 || (useKey && !keyPath.trim())">
          {{ busy ? "Writing…" : "Save" }}
        </button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.access-dialog {
  width: 540px;
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

.small {
  font-size: 12px;
}

.muted {
  color: var(--text-muted);
  font-weight: 400;
}
</style>
