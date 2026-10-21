<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { ask } from "@tauri-apps/plugin-dialog";
import { getHeadCommit } from "../api/commit";
import { toAppError } from "../api/errors";
import type { HeadCommit } from "../types/commit";

// Commit message + Amend option. The parent owns both values (it clears them
// after a successful commit) and runs the actual commit when we emit.
const props = defineProps<{
  repoId: string;
  stagedCount: number;
  busy: boolean;
}>();

const message = defineModel<string>("message", { required: true });
const amend = defineModel<boolean>("amend", { required: true });

const emit = defineEmits<{
  commit: [];
}>();

// Loaded only when Amend is ticked: finding out whether HEAD was pushed
// walks the history, which can take a moment in large repositories.
const head = ref<HeadCommit | null>(null);
const note = ref<string | null>(null);
// The message we filled in for amend; removed again if Amend is unticked unchanged.
let prefilled: string | null = null;

const canCommit = computed(
  () => !props.busy && message.value.trim() !== "" && (amend.value || props.stagedCount > 0),
);

const hint = computed(() => {
  if (props.stagedCount > 0) {
    return `${props.stagedCount} staged ${props.stagedCount === 1 ? "file" : "files"}`;
  }
  return amend.value ? "Nothing staged: only the message changes" : "Nothing staged";
});

watch(amend, async (on) => {
  note.value = null;
  if (!on) {
    if (prefilled !== null && message.value === prefilled) {
      message.value = "";
    }
    prefilled = null;
    head.value = null;
    return;
  }
  try {
    const last = await getHeadCommit(props.repoId);
    if (!amend.value) {
      return; // unticked again while we were waiting
    }
    if (last === null) {
      amend.value = false;
      note.value = "There is no commit to amend yet.";
      return;
    }
    head.value = last;
    if (message.value.trim() === "") {
      message.value = last.message;
      prefilled = last.message;
    }
  } catch (err) {
    amend.value = false;
    note.value = toAppError(err).message;
  }
});

async function submit() {
  if (!canCommit.value) {
    return;
  }
  // Not destructive locally, but the old commit is already on the server:
  // the next push would have to overwrite it. Make that a conscious choice.
  if (amend.value && head.value?.pushed) {
    const confirmed = await ask(
      "The last commit has already been pushed.\n\nAmending it rewrites published history: the next push will need --force, and anyone who pulled it will have to fix their copy.",
      { title: "Amend Pushed Commit", kind: "warning", okLabel: "Amend", cancelLabel: "Cancel" },
    );
    if (!confirmed) {
      return;
    }
  }
  emit("commit");
}
</script>

<template>
  <div class="commit-box">
    <!-- Cmd+Enter (macOS) / Ctrl+Enter commits, like in most editors. -->
    <textarea
      v-model="message"
      rows="3"
      placeholder="Commit message (first line = summary)"
      spellcheck="false"
      @keydown.meta.enter.prevent="submit"
      @keydown.ctrl.enter.prevent="submit"
    ></textarea>
    <div class="actions">
      <label class="amend">
        <input v-model="amend" type="checkbox" :disabled="busy" />
        Amend last commit
      </label>
      <span class="muted">{{ hint }}</span>
      <span class="spacer"></span>
      <button class="primary" :disabled="!canCommit" title="Commit (⌘/Ctrl+Enter)" @click="submit">
        {{ amend ? "Amend" : "Commit" }}
      </button>
    </div>
    <p v-if="note" class="muted note">{{ note }}</p>
    <p v-else-if="amend && head?.pushed" class="warn note">This commit is already pushed.</p>
  </div>
</template>

<style scoped>
.commit-box {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding-top: 8px;
  border-top: 1px solid var(--border);
}

textarea {
  resize: vertical;
  min-height: 52px;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
}

.actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.amend {
  display: flex;
  align-items: center;
  gap: 4px;
}

.spacer {
  flex: 1;
}

.muted {
  color: var(--text-muted);
}

.warn {
  color: var(--warning);
}

.note {
  margin: 0;
}
</style>
