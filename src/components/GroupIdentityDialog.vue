<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import { message as showMessage } from "@tauri-apps/plugin-dialog";
import { getComputerIdentity } from "../api/identity";
import { toAppError } from "../api/errors";
import { useDialogStore } from "../stores/dialogs";
import { useRepoStore } from "../stores/repos";
import { useApplyTarget } from "./applyTarget";

// "Commit Identity" of a group (step 14b) or of the repositories picked in the
// sidebar (step 35c): a name and email written into their .git/config (for a
// group: also its subgroups' repositories), so git in a terminal uses them
// too. Two ways:
//   - Write for all repositories: also over the identity some already have;
//   - Only repositories without their own: those keep theirs.
const dialogs = useDialogStore();
const store = useRepoStore();

const name = ref("");
const email = ref("");
const all = ref(false);
const message = ref<string | null>(null);
const busy = ref(false);
const nameInput = ref<HTMLInputElement | null>(null);

const { group, repoIds, label, repoCount, where, ready } = useApplyTarget(() => dialogs.groupIdentityRequest);

watch(
  () => dialogs.groupIdentityRequest,
  async (request) => {
    message.value = null;
    all.value = false;
    if (request === null || !ready.value) {
      return;
    }
    // A group's last identity, or else the computer's as a starting point.
    const last = group.value?.identity;
    name.value = last?.name ?? "";
    email.value = last?.email ?? "";
    await nextTick();
    nameInput.value?.focus();
    if (last === undefined) {
      try {
        const computer = await getComputerIdentity();
        name.value ||= computer.inheritedName ?? "";
        email.value ||= computer.inheritedEmail ?? "";
      } catch {
        // Only a suggestion: leave the fields empty.
      }
    }
  },
);

async function onSubmit() {
  if (!ready.value || busy.value) {
    return;
  }
  const title = `Commit Identity · ${label.value}`;
  busy.value = true;
  message.value = null;
  try {
    const result =
      group.value !== null
        ? await store.setGroupIdentity(group.value.id, name.value, email.value, all.value)
        : await store.setReposIdentity(repoIds.value ?? [], name.value, email.value, all.value);
    dialogs.closeGroupIdentity();
    const kept = result.total - result.changed - result.failed.length;
    const lines = [`Written to ${result.changed} of ${result.total} ${result.total === 1 ? "repository" : "repositories"}.`];
    if (kept > 0) {
      lines.push(`${kept} kept ${kept === 1 ? "its" : "their"} own (or already had this one).`);
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
  <div v-if="ready" class="backdrop" @mousedown.self="dialogs.closeGroupIdentity()">
    <form class="dialog identity-dialog" @submit.prevent="onSubmit" @keydown.esc="dialogs.closeGroupIdentity()">
      <h2>Commit Identity · {{ label }}</h2>
      <p class="muted">
        Written into the {{ repoCount }} {{ repoCount === 1 ? "repository" : "repositories" }} {{ where }} (their
        <code>.git/config</code>), so git in a terminal uses it too.
      </p>

      <label class="field">
        <span>Name</span>
        <input ref="nameInput" v-model="name" placeholder="Your Name" spellcheck="false" />
      </label>
      <label class="field">
        <span>Email</span>
        <input v-model="email" placeholder="you@example.com" spellcheck="false" />
      </label>

      <label class="option">
        <input v-model="all" type="radio" :value="false" />
        <span>
          <strong>Only repositories without their own setting</strong>
          <span class="muted detail">Repositories that already have a name or email keep theirs.</span>
        </span>
      </label>
      <label class="option">
        <input v-model="all" type="radio" :value="true" />
        <span>
          <strong>Write for all repositories</strong>
          <span class="muted detail">Also replaces the name and email some repositories already have.</span>
        </span>
      </label>

      <p v-if="message" class="err">{{ message }}</p>
      <div class="buttons">
        <button type="button" @click="dialogs.closeGroupIdentity()">Cancel</button>
        <button type="submit" class="primary" :disabled="busy || repoCount === 0 || !name.trim() || !email.trim()">
          {{ busy ? "Writing…" : "Save" }}
        </button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.identity-dialog {
  width: 480px;
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

.muted {
  color: var(--text-muted);
}
</style>
