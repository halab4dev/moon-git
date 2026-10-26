<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";
import { ask } from "@tauri-apps/plugin-dialog";
import { getHeadCommit } from "../api/commit";
import { toAppError } from "../api/errors";
import type { HeadCommit } from "../types/commit";

// The Commit dialog (toolbar → Commit…): message, Amend option, then Commit
// or Commit and Push. The parent owns the message and the options, so a
// draft survives closing the dialog; it clears them after a commit.
const props = defineProps<{
  repoId: string;
  stagedCount: number;
  /** A merge waits for its commit: offer merge commit / simple commit instead of Amend. */
  merging: boolean;
  /** The branch can't be pushed (e.g. HEAD is detached): only Commit. */
  pushBlocker: string | null;
  /**
   * Runs the commit (`push`: then push it). Returns an error message to show
   * here, or null when done (the parent closes the dialog).
   */
  submit: (push: boolean) => Promise<string | null>;
}>();

const message = defineModel<string>("message", { required: true });
const amend = defineModel<boolean>("amend", { required: true });
// During a merge: false = the merge commit (two parents), true = an ordinary commit.
const simple = defineModel<boolean>("simple", { required: true });

const emit = defineEmits<{
  close: [];
}>();

const busy = ref(false);
const error = ref<string | null>(null);
// Loaded only when Amend is ticked: finding out whether HEAD was pushed
// walks the history, which can take a moment in large repositories.
const head = ref<HeadCommit | null>(null);
const note = ref<string | null>(null);
// The message we filled in for amend; removed again if Amend is unticked unchanged.
let prefilled: string | null = null;
const textarea = ref<HTMLTextAreaElement | null>(null);

onMounted(async () => {
  await nextTick();
  textarea.value?.focus();
});

const canCommit = computed(
  () => !busy.value && message.value.trim() !== "" && (amend.value || props.stagedCount > 0),
);

const commitLabel = computed(() => {
  if (props.merging) {
    return simple.value ? "Commit" : "Commit Merge";
  }
  return amend.value ? "Amend" : "Commit";
});

// An amended commit that was already pushed can't be pushed normally
// any more: that needs Force Push, which stays a separate, explicit step.
const andPushBlocker = computed(() => {
  if (props.pushBlocker !== null) {
    return props.pushBlocker;
  }
  if (amend.value && head.value?.pushed) {
    return "The amended commit replaces a pushed one: use Force Push (▾ next to Push) afterwards";
  }
  return null;
});

const hint = computed(() => {
  if (props.stagedCount > 0) {
    return `${props.stagedCount} staged ${props.stagedCount === 1 ? "file" : "files"}`;
  }
  return amend.value ? "Nothing staged: only the message changes" : "Nothing staged: stage files first";
});

// immediate: Amend may still be ticked from the last time the dialog was open.
watch(
  amend,
  async (on) => {
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
  },
  { immediate: true },
);

async function commit(push: boolean) {
  if (!canCommit.value || (push && andPushBlocker.value !== null)) {
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
  busy.value = true;
  error.value = null;
  try {
    error.value = await props.submit(push);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="backdrop" @mousedown.self="emit('close')">
    <form class="dialog commit-dialog" @submit.prevent="commit(false)" @keydown.esc="emit('close')">
      <h2>Commit</h2>
      <!-- Cmd+Enter (macOS) / Ctrl+Enter commits, like in most editors. -->
      <textarea
        ref="textarea"
        v-model="message"
        rows="7"
        placeholder="Commit message (first line = summary)"
        spellcheck="false"
        @keydown.meta.enter.prevent="commit(false)"
        @keydown.ctrl.enter.prevent="commit(false)"
      ></textarea>
      <div class="options">
        <template v-if="merging">
          <!-- :value="false"/"true": the radio sets the boolean model, not a string. -->
          <label class="choice" title="Two parents: the history shows that the other branch was merged">
            <input v-model="simple" type="radio" :value="false" :disabled="busy" />
            Merge commit
          </label>
          <label class="choice" title="One parent, like a squash: the same changes, but no link to the other branch">
            <input v-model="simple" type="radio" :value="true" :disabled="busy" />
            Simple commit
          </label>
        </template>
        <label v-else class="choice">
          <input v-model="amend" type="checkbox" :disabled="busy" />
          Amend last commit
        </label>
        <span class="muted">{{ hint }}</span>
      </div>
      <p v-if="note" class="muted">{{ note }}</p>
      <p v-else-if="amend && head?.pushed" class="warn">This commit is already pushed.</p>
      <p v-if="error" class="err">{{ error }}</p>
      <div class="buttons">
        <button type="button" @click="emit('close')">Cancel</button>
        <button
          type="button"
          :disabled="!canCommit || andPushBlocker !== null"
          :title="andPushBlocker ?? 'Commit, then push the branch to the server'"
          @click="commit(true)"
        >
          {{ commitLabel }} and Push
        </button>
        <button type="submit" class="primary" :disabled="!canCommit" title="Commit (⌘/Ctrl+Enter)">
          {{ commitLabel }}
        </button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.commit-dialog {
  width: 560px;
}

textarea {
  resize: vertical;
  min-height: 100px;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
}

.options {
  display: flex;
  align-items: center;
  gap: 14px;
}

.choice {
  display: flex;
  align-items: center;
  gap: 4px;
}

.muted {
  color: var(--text-muted);
}

.warn {
  color: var(--warning);
}
</style>
