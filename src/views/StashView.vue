<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Menu } from "@tauri-apps/api/menu";
import { ask } from "@tauri-apps/plugin-dialog";
import CommitDetails from "../components/CommitDetails.vue";
import { useSplitRatio } from "../composables/useSplitRatio";
import { listStashes, runStashAction } from "../api/stash";
import { toAppError } from "../api/errors";
import type { AppError } from "../types/error";
import type { StashAction, StashEntry } from "../types/stash";
import type { RepoStatus } from "../types/status";

// The Stashes tab: every stash on top, the changes of the selected one below.
const props = defineProps<{
  repoId: string;
  /** Changes after every new status (e.g. a stash was made): reload then. */
  version: number;
  /** Another git action is running in this repo: wait. */
  busy: boolean;
}>();

const emit = defineEmits<{
  /** Apply/pop/drop worked: the new status. */
  statusChanged: [status: RepoStatus];
  /** It failed (e.g. conflicts). RepoView reloads the status and shows the error. */
  failed: [error: AppError];
}>();

// True while a stash action runs, so RepoView blocks other git actions
// (same idea as SyncButtons' "locked").
const locked = defineModel<boolean>("locked", { required: true });

const stashes = ref<StashEntry[]>([]);
const loading = ref(false);
const error = ref<AppError | null>(null);
// By id, not index: the index of a stash shifts when a new one is made.
const selectedId = ref<string | null>(null);
// Which part of the selected stash is shown below.
const part = ref<"tracked" | "untracked">("tracked");

const selected = computed(() => stashes.value.find((s) => s.id === selectedId.value) ?? null);

// A stash with untracked files is two commits: the stash itself (tracked
// files, compared with the commit it was made on) and a separate commit
// holding only the untracked files (all shown as "added").
const shownCommit = computed(() => {
  const stash = selected.value;
  if (stash === null) {
    return null;
  }
  return part.value === "untracked" && stash.untracked !== null ? stash.untracked : stash.id;
});

let request = 0;

async function load() {
  const current = ++request;
  loading.value = true;
  try {
    const result = await listStashes(props.repoId);
    if (current !== request) {
      return;
    }
    stashes.value = result;
    error.value = null;
    // Keep the selection if that stash still exists, else take the newest.
    if (!result.some((s) => s.id === selectedId.value)) {
      selectedId.value = result[0]?.id ?? null;
    }
  } catch (err) {
    if (current === request) {
      error.value = toAppError(err);
    }
  } finally {
    if (current === request) {
      loading.value = false;
    }
  }
}

watch(() => props.version, load, { immediate: true });

watch(selectedId, () => {
  part.value = "tracked";
});

// ↑/↓ in the list.
function move(step: number) {
  const list = stashes.value;
  if (list.length === 0) {
    return;
  }
  const current = list.findIndex((s) => s.id === selectedId.value);
  const next = Math.min(list.length - 1, Math.max(0, current + step));
  selectedId.value = list[next].id;
  document.querySelector(`li[data-stash="${list[next].id}"]`)?.scrollIntoView({ block: "nearest" });
}

// ---- apply / pop / drop ----

async function perform(stash: StashEntry, action: StashAction) {
  if (props.busy) {
    return;
  }
  if (action === "drop" && !(await confirmDrop(stash))) {
    return;
  }
  locked.value = true;
  let failure: AppError | null = null;
  try {
    emit("statusChanged", await runStashAction(props.repoId, stash.id, action));
  } catch (err) {
    failure = toAppError(err);
  } finally {
    locked.value = false;
  }
  // After unlocking: RepoView reloads the status first, which it can't while locked.
  if (failure !== null) {
    emit("failed", failure);
  }
}

// Dropping deletes the changes for good (CLAUDE.md: confirm destructive actions).
function confirmDrop(stash: StashEntry): Promise<boolean> {
  return ask(`Drop ${stashName(stash)}?\n\n"${stash.message}"\n\nThe changes in this stash will be lost.`, {
    title: "Drop Stash",
    kind: "warning",
    okLabel: "Drop",
    cancelLabel: "Cancel",
  });
}

async function showMenu(stash: StashEntry) {
  selectedId.value = stash.id;
  const enabled = !props.busy;
  const menu = await Menu.new({
    items: [
      { text: "Apply", enabled, action: () => void perform(stash, "apply") },
      { text: "Pop (Apply and Drop)", enabled, action: () => void perform(stash, "pop") },
      { item: "Separator" },
      { text: "Drop…", enabled, action: () => void perform(stash, "drop") },
    ],
  });
  await menu.popup();
}

// "stash@{0}". (Written here: braces inside {{ }} in the template are confusing.)
function stashName(stash: StashEntry): string {
  return `stash@{${stash.index}}`;
}

const dateFormat = new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" });

function formatTime(seconds: number): string {
  return dateFormat.format(new Date(seconds * 1000));
}

const { ratio: listRatio, startResize } = useSplitRatio("split", "moongit.stashes.listRatio", 0.3);
</script>

<template>
  <div class="stashes">
    <p v-if="error" class="err message">{{ error.message }}</p>
    <div class="stash-toolbar">
      <button
        :disabled="busy || !selected"
        title="Put the changes of the selected stash back into the working tree; keep the stash"
        @click="selected && perform(selected, 'apply')"
      >
        Apply
      </button>
      <button
        :disabled="busy || !selected"
        title="Apply the selected stash, then drop it (it is kept if there are conflicts)"
        @click="selected && perform(selected, 'pop')"
      >
        Pop
      </button>
      <button :disabled="busy || !selected" title="Delete the selected stash" @click="selected && perform(selected, 'drop')">
        Drop…
      </button>
    </div>
    <div ref="split" class="split">
      <div class="list" :style="{ flexBasis: `${listRatio * 100}%` }">
        <p v-if="stashes.length === 0 && !loading" class="muted">
          No stashes. Use "Stash…" to put your local changes aside.
        </p>
        <ul v-else tabindex="0" @keydown.up.prevent="move(-1)" @keydown.down.prevent="move(1)">
          <li
            v-for="stash in stashes"
            :key="stash.id"
            :data-stash="stash.id"
            :class="{ selected: stash.id === selectedId }"
            @click="selectedId = stash.id"
            @contextmenu.prevent="showMenu(stash)"
          >
            <span class="ref">{{ stashName(stash) }}</span>
            <span class="text">{{ stash.message }}</span>
            <span v-if="stash.untracked" class="tag" title="Includes untracked files">+ untracked</span>
            <span class="time">{{ formatTime(stash.time) }}</span>
          </li>
        </ul>
      </div>
      <div class="splitter" title="Drag to resize" @mousedown="startResize"></div>
      <div class="bottom">
        <nav v-if="selected?.untracked" class="parts">
          <button :class="{ active: part === 'tracked' }" @click="part = 'tracked'">Changes</button>
          <button :class="{ active: part === 'untracked' }" @click="part = 'untracked'">Untracked Files</button>
        </nav>
        <CommitDetails class="details" :repo-id="repoId" :commit-id="shownCommit" hide-parents />
      </div>
    </div>
  </div>
</template>

<style scoped>
.stashes {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.stash-toolbar {
  display: flex;
  gap: 8px;
  margin-bottom: 6px;
}

.split {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.list {
  flex-grow: 0;
  flex-shrink: 0;
  min-height: 0;
  overflow-y: auto;
}

ul {
  margin: 0;
  padding: 0;
  list-style: none;
}

ul:focus {
  outline: none;
}

li {
  display: flex;
  gap: 10px;
  align-items: baseline;
  padding: 3px 6px;
  white-space: nowrap;
  cursor: default;
  user-select: none;
}

li:hover {
  background: var(--bg-sidebar);
}

li.selected {
  background: var(--accent);
  color: #fff;
}

li.selected .ref,
li.selected .time,
li.selected .tag {
  color: #fff;
}

.ref {
  flex-shrink: 0;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
  color: var(--text-muted);
}

.text {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tag {
  flex-shrink: 0;
  font-size: 11px;
  color: var(--accent);
}

.time {
  flex-shrink: 0;
  color: var(--text-muted);
}

.splitter {
  flex: 0 0 6px;
  cursor: row-resize;
}

.splitter:hover {
  background: var(--border);
}

.bottom {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.parts {
  display: flex;
  gap: 4px;
  margin-bottom: 4px;
}

.parts button {
  padding: 2px 10px;
  font-size: 12px;
}

.parts button.active {
  background: var(--accent);
  color: #fff;
  border-color: var(--accent);
}

.details {
  flex: 1;
  min-height: 0;
}

.muted {
  color: var(--text-muted);
}

.message {
  white-space: pre-wrap;
}
</style>
