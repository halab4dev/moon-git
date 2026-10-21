<script setup lang="ts">
import { ref } from "vue";
import { checkRepo } from "../api/repo";
import { toAppError } from "../api/errors";
import type { RepoInfo } from "../types/repo";
import type { AppError } from "../types/error";

const path = ref("");
const checking = ref(false);
const repo = ref<RepoInfo | null>(null);
const error = ref<AppError | null>(null);

async function onCheck() {
  checking.value = true;
  repo.value = null;
  error.value = null;
  try {
    repo.value = await checkRepo(path.value.trim());
  } catch (err) {
    error.value = toAppError(err);
  } finally {
    checking.value = false;
  }
}
</script>

<template>
  <section class="welcome">
    <h1>MoonGit</h1>
    <p class="muted">Select a repository on the left, or add one (coming soon).</p>

    <!-- Temporary test form for the check_repo command; replaced by "Add repository" in step 3. -->
    <form class="demo" @submit.prevent="onCheck">
      <input v-model="path" placeholder="/path/to/folder" />
      <button type="submit" :disabled="checking || !path.trim()">Check repository</button>
    </form>

    <p v-if="repo" class="ok">
      Git repository <strong>{{ repo.name }}</strong> <span class="muted">— {{ repo.root }}</span>
    </p>
    <p v-if="error" class="err">
      {{ error.message }} <span class="muted">({{ error.kind }})</span>
    </p>
  </section>
</template>

<style scoped>
.welcome {
  padding: 24px 32px;
}

.muted {
  color: var(--text-muted);
}

.demo {
  display: flex;
  gap: 8px;
  margin-top: 16px;
}

.demo input {
  flex: 1;
  max-width: 480px;
  padding: 4px 8px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--bg);
}

.demo button {
  padding: 4px 12px;
  border: 1px solid var(--accent);
  border-radius: 4px;
  background: var(--accent);
  color: #fff;
  cursor: pointer;
}

.demo button:disabled {
  opacity: 0.5;
  cursor: default;
}

.ok {
  color: var(--success);
}

.err {
  color: var(--danger);
}
</style>
