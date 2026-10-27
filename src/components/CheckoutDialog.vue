<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";
import { listBranches } from "../api/branch";
import { toAppError } from "../api/errors";
import { filterBranches, type BranchChoice } from "./branchFilter";
import type { BranchList } from "../types/branches";

// Branch → Checkout… (step 31b): type to filter, ↑/↓ to move, Enter to
// check out. A remote branch gets a local branch tracking it, as when it
// is double-clicked in the Branches panel.
const props = defineProps<{
  repoId: string;
  /** Checks out `fullName`. Returns an error message to show here, or null when done. */
  submit: (fullName: string) => Promise<string | null>;
}>();

const emit = defineEmits<{
  close: [];
}>();

const branches = ref<BranchList | null>(null);
const query = ref("");
const error = ref<string | null>(null);
const busy = ref(false);
// Index in `choices` of the highlighted row.
const highlighted = ref(0);
const input = ref<HTMLInputElement | null>(null);
const listElement = ref<HTMLElement | null>(null);

onMounted(async () => {
  await nextTick();
  input.value?.focus();
  try {
    branches.value = await listBranches(props.repoId);
    highlightFirst();
  } catch (err) {
    error.value = toAppError(err).message;
  }
});

const choices = computed(() => (branches.value === null ? [] : filterBranches(branches.value, query.value)));

// Where the "Remote Branches" heading goes (index of the first remote row).
const firstRemote = computed(() => choices.value.findIndex((choice) => choice.kind === "remote"));

// The current branch can't be checked out again: start on the first other one.
function highlightFirst() {
  const first = choices.value.findIndex((choice) => !choice.current);
  highlighted.value = Math.max(first, 0);
}

watch(query, () => {
  error.value = null;
  highlightFirst();
});

async function move(step: number) {
  const list = choices.value;
  let next = highlighted.value;
  // Skip the current branch.
  do {
    next += step;
  } while (next >= 0 && next < list.length && list[next].current);
  if (next < 0 || next >= list.length) {
    return;
  }
  highlighted.value = next;
  await nextTick();
  listElement.value?.querySelector(".highlighted")?.scrollIntoView({ block: "nearest" });
}

async function checkout(choice: BranchChoice | undefined) {
  if (choice === undefined || choice.current || busy.value) {
    return;
  }
  busy.value = true;
  error.value = null;
  try {
    error.value = await props.submit(choice.fullName);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="backdrop" @mousedown.self="emit('close')">
    <form
      class="dialog checkout-dialog"
      @submit.prevent="checkout(choices[highlighted])"
      @keydown.esc="emit('close')"
      @keydown.down.prevent="move(1)"
      @keydown.up.prevent="move(-1)"
    >
      <h2>Check Out Branch</h2>
      <input ref="input" v-model="query" placeholder="Type to filter, e.g. &quot;fea log&quot;" spellcheck="false" />
      <ul ref="listElement" class="choices">
        <template v-for="(choice, index) in choices" :key="choice.fullName">
          <li v-if="index === 0 && choice.kind === 'local'" class="heading">Local Branches</li>
          <li v-if="index === firstRemote" class="heading">Remote Branches</li>
          <li
            class="choice"
            :class="{ highlighted: index === highlighted, current: choice.current }"
            :title="choice.current ? 'Already checked out' : choice.kind === 'remote' ? 'Creates a local branch that tracks it' : ''"
            @mousemove="!choice.current && (highlighted = index)"
            @click="checkout(choice)"
          >
            <span class="mark">{{ choice.current ? "✓" : "" }}</span>
            <span class="label">{{ choice.label }}</span>
            <span class="detail">{{ choice.detail }}</span>
          </li>
        </template>
        <li v-if="branches && choices.length === 0" class="empty">No branch matches "{{ query }}".</li>
        <li v-if="!branches && !error" class="empty">Loading…</li>
      </ul>
      <p v-if="error" class="err">{{ error }}</p>
      <div class="buttons">
        <button type="button" @click="emit('close')">Cancel</button>
        <button type="submit" class="primary" :disabled="busy || !choices[highlighted] || choices[highlighted].current">
          Check Out
        </button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.checkout-dialog {
  width: 480px;
}

.choices {
  height: 300px;
  margin: 0;
  padding: 0;
  overflow-y: auto;
  list-style: none;
  border: 1px solid var(--border);
  border-radius: 4px;
}

.heading {
  position: sticky;
  top: 0;
  padding: 4px 8px;
  background: var(--bg-sidebar);
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.choice {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 3px 8px;
  cursor: default;
  white-space: nowrap;
}

.choice.highlighted {
  background: var(--accent);
  color: #fff;
}

.choice.current {
  font-weight: 600;
}

.mark {
  width: 12px;
  color: var(--success);
}

.label {
  overflow: hidden;
  text-overflow: ellipsis;
}

.detail {
  margin-left: auto;
  color: var(--text-muted);
  font-size: 11px;
}

.highlighted .detail,
.highlighted .mark {
  color: #fff;
}

.empty {
  padding: 8px;
  color: var(--text-muted);
}
</style>
