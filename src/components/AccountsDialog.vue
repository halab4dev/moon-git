<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import { ask } from "@tauri-apps/plugin-dialog";
import { listAccounts, removeAccount } from "../api/account";
import { toAppError } from "../api/errors";
import { useDialogStore } from "../stores/dialogs";
import type { Account } from "../types/account";

// MoonGit → Accounts… (step 13c): the saved logins. Only the server and the
// user name are shown; the token stays in the keychain (no command returns it).
const dialogs = useDialogStore();

const accounts = ref<Account[] | null>(null);
const error = ref<string | null>(null);
const busy = ref(false);
// Focused when opened, so Esc closes it.
const box = ref<HTMLElement | null>(null);

watch(
  () => dialogs.accountsOpen,
  async (open) => {
    if (!open) {
      return;
    }
    accounts.value = null;
    error.value = null;
    await nextTick();
    box.value?.focus();
    try {
      accounts.value = await listAccounts();
    } catch (err) {
      error.value = toAppError(err).message;
    }
  },
);

// The first account of a server is the one git uses for it.
function usedForServer(account: Account, index: number): boolean {
  return accounts.value?.findIndex((a) => a.host.toLowerCase() === account.host.toLowerCase()) === index;
}

async function remove(account: Account) {
  const confirmed = await ask(
    `Remove the account "${account.username}" for ${account.host}?\n\nIts token is deleted from the keychain. The next time a repository on ${account.host} needs it, MoonGit asks you to sign in again.`,
    { title: "Remove Account", kind: "warning", okLabel: "Remove", cancelLabel: "Cancel" },
  );
  if (!confirmed) {
    return;
  }
  busy.value = true;
  error.value = null;
  try {
    accounts.value = await removeAccount(account.id);
  } catch (err) {
    error.value = toAppError(err).message;
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div v-if="dialogs.accountsOpen" class="backdrop" @mousedown.self="dialogs.closeAccounts()">
    <div ref="box" class="dialog accounts-dialog" tabindex="-1" @keydown.esc="dialogs.closeAccounts()">
      <h2>Accounts</h2>
      <p class="muted">
        Logins for HTTPS servers. The password or token of each one is kept in the system keychain. MoonGit adds one
        when a server asks you to sign in.
      </p>

      <table v-if="accounts && accounts.length > 0">
        <thead>
          <tr>
            <th>Server</th>
            <th>User name</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(account, index) in accounts" :key="account.id">
            <td>{{ account.host }}</td>
            <td>
              {{ account.username }}
              <span v-if="!usedForServer(account, index)" class="muted" title="Another account of this server is used first">
                (not used)
              </span>
            </td>
            <td class="actions">
              <button :disabled="busy" @click="remove(account)">Remove…</button>
            </td>
          </tr>
        </tbody>
      </table>
      <p v-else-if="accounts" class="muted empty">
        No saved accounts. Repositories over SSH, and HTTPS logins already saved by git on this computer, don't need
        one.
      </p>
      <p v-else-if="!error" class="muted">Loading…</p>

      <p v-if="error" class="err">{{ error }}</p>
      <div class="buttons">
        <button class="primary" @click="dialogs.closeAccounts()">Close</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.accounts-dialog {
  width: 520px;
  outline: none;
}

table {
  width: 100%;
  border-collapse: collapse;
}

th {
  padding: 4px 6px;
  text-align: left;
  font-weight: 600;
  color: var(--text-muted);
  border-bottom: 1px solid var(--border);
}

td {
  padding: 5px 6px;
  border-bottom: 1px solid var(--border);
}

.actions {
  text-align: right;
}

.empty {
  padding: 12px 0;
}

.muted {
  color: var(--text-muted);
}
</style>
