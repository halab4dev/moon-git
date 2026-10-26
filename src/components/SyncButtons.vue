<script setup lang="ts">
import { computed, ref } from "vue";
import { Menu } from "@tauri-apps/api/menu";
import { ask, message } from "@tauri-apps/plugin-dialog";
import AppIcon from "./AppIcon.vue";
import { fetchRepo } from "../api/fetch";
import { cancelOperation, onGitProgress } from "../api/operations";
import { getStatus } from "../api/status";
import { forcePushPreview, forcePushRepo, getPullDefault, listRemotes, pullRepo, pushRepo } from "../api/sync";
import { toAppError } from "../api/errors";
import type { AppError } from "../types/error";
import type { BranchInfo } from "../types/status";
import type { ForcePushPreview, PullMode } from "../types/sync";

// Fetch / Pull / Push. One at a time; while one runs, the buttons turn into
// a progress line (+ Cancel while git talks to the server).
const props = defineProps<{
  repoId: string;
  branch: BranchInfo;
  /** The parent is staging/committing. */
  busy: boolean;
}>();

/** True while a pull may change files: the parent blocks staging and committing. */
const locked = defineModel<boolean>("locked", { required: true });

const emit = defineEmits<{
  /** Always emitted at the end (also after an error): the status must be reloaded. */
  finished: [error: AppError | null];
}>();

const running = ref(false);
// Set while a cancellable git command runs.
const opId = ref<string | null>(null);
const progress = ref("");
const cancelling = ref(false);
// Short result shown next to the buttons, e.g. "Already up to date".
const note = ref<string | null>(null);

const tracking = computed(() => props.branch.upstream !== null && !props.branch.upstreamGone);

const pullBlocker = computed(() => {
  if (props.branch.name === null) return "HEAD is detached";
  if (props.branch.upstream === null) return "This branch has no upstream";
  if (props.branch.upstreamGone) return "The upstream branch was deleted on the server";
  return null;
});

const pushBlocker = computed(() => {
  if (props.branch.name === null) return "HEAD is detached";
  if (props.branch.commit === null) return "Nothing committed yet";
  if (tracking.value && props.branch.ahead === 0) return "Nothing to push";
  return null;
});

// Force push replaces the server branch, so it needs one to replace.
const forcePushBlocker = computed(() => {
  if (props.branch.name === null) return "HEAD is detached";
  if (props.branch.upstream === null) return "This branch is not on the server yet: use Push";
  if (props.branch.upstreamGone) return "The upstream branch was deleted on the server: use Push";
  return null;
});

// ---- running git with progress ----

/**
 * Runs one long git command. Resolves true when it finished, false when
 * the user cancelled it; other errors are thrown.
 */
async function withProgress(label: string, start: (id: string) => Promise<void>): Promise<boolean> {
  const id = crypto.randomUUID();
  opId.value = id;
  cancelling.value = false;
  progress.value = label;
  // Listen before starting so no early progress line is missed.
  const unlisten = await onGitProgress((line) => {
    if (line.opId === id) {
      progress.value = line.phase !== null ? `${line.phase} ${line.percent}%` : line.line;
    }
  });
  try {
    await start(id);
    return true;
  } catch (err) {
    if (toAppError(err).kind === "cancelled") {
      return false;
    }
    throw err;
  } finally {
    unlisten();
    opId.value = null;
  }
}

// Shared frame of the three actions: one at a time, report at the end.
async function perform(action: () => Promise<void>) {
  if (running.value) {
    return;
  }
  running.value = true;
  note.value = null;
  let error: AppError | null = null;
  try {
    await action();
  } catch (err) {
    error = toAppError(err);
  } finally {
    running.value = false;
    locked.value = false;
    emit("finished", error);
  }
}

async function cancel() {
  if (opId.value === null) {
    return;
  }
  cancelling.value = true;
  try {
    await cancelOperation(opId.value);
  } catch (err) {
    cancelling.value = false;
    note.value = toAppError(err).message;
  }
}

// ---- the actions ----

function fetchNow() {
  return perform(async () => {
    await withProgress("Fetching…", (id) => fetchRepo(id, props.repoId));
  });
}

function pull() {
  return perform(async () => {
    locked.value = true;
    // 1. Download (the slow, network part: progress + Cancel).
    if (!(await withProgress("Fetching…", (id) => fetchRepo(id, props.repoId)))) {
      return;
    }
    // 2. Compare with the freshly fetched upstream.
    const { ahead, behind } = (await getStatus(props.repoId)).branch;
    if (behind === 0) {
      note.value = "Already up to date";
      return;
    }
    // 3. Only behind: fast-forward, nothing to decide. Both changed: ask.
    let mode: PullMode = "fastForward";
    if (ahead > 0) {
      progress.value = "Pull: choose merge or rebase…";
      const chosen = await chooseMode(ahead, behind, await getPullDefault(props.repoId));
      if (chosen === null) {
        return;
      }
      mode = chosen;
    }
    // 4. Local merge/rebase: quick, and not safe to interrupt, so no Cancel.
    progress.value = mode === "rebase" ? "Rebasing…" : "Merging…";
    await pullRepo(props.repoId, mode);
  });
}

function push() {
  return perform(async () => {
    let newUpstream: string | null = null;
    // First push of this branch: say where it goes before creating it there.
    if (props.branch.upstream === null) {
      const remotes = await listRemotes(props.repoId);
      if (remotes.length === 0) {
        const noRemote: AppError = { kind: "invalidInput", message: "This repository has no remote to push to." };
        throw noRemote;
      }
      const remote = remotes.includes("origin") ? "origin" : remotes[0];
      const name = props.branch.name;
      const confirmed = await ask(
        `"${name}" is not on the server yet.\n\nPush it to ${remote}? It will be created as ${remote}/${name}, and this branch will track it.`,
        { title: "Push New Branch", kind: "info", okLabel: "Push", cancelLabel: "Cancel" },
      );
      if (!confirmed) {
        return;
      }
      newUpstream = remote;
    }
    await withProgress("Pushing…", (id) => pushRepo(id, props.repoId, newUpstream));
  });
}

// ---- force push (step 27) ----

async function pushMenu() {
  if (running.value) {
    return;
  }
  const blocker = forcePushBlocker.value;
  const menu = await Menu.new({
    items: [
      {
        text: blocker === null ? "Force Push…" : `Force Push… (${blocker})`,
        enabled: blocker === null,
        action: () => void forcePush(),
      },
    ],
  });
  await menu.popup();
}

function forcePush() {
  return perform(async () => {
    // Based on the last fetch; the lease makes git check it on the server.
    const preview = await forcePushPreview(props.repoId);
    if (preview.lostCount === 0) {
      await message(
        `${preview.upstream} has no commits that "${preview.branch}" doesn't have, so nothing would be overwritten. Use Push instead.`,
        { title: "Force Push Not Needed", kind: "info" },
      );
      return;
    }
    const confirmed = await ask(forcePushQuestion(preview), {
      title: "Force Push",
      kind: "warning",
      okLabel: "Force Push",
      cancelLabel: "Cancel",
    });
    if (!confirmed) {
      return;
    }
    if (await withProgress("Force pushing…", (id) => forcePushRepo(id, props.repoId, preview.expected))) {
      note.value = "Force pushed";
    }
  });
}

function forcePushQuestion(preview: ForcePushPreview): string {
  const lines = preview.lost.map((commit) => `  • ${commit.id.slice(0, 8)} ${commit.subject}`);
  const more = preview.lostCount - preview.lost.length;
  if (more > 0) {
    lines.push(`  … and ${more} more`);
  }
  return [
    `Replace ${preview.upstream} with your local "${preview.branch}"?`,
    "",
    `${preview.lostCount} commit(s) on ${preview.upstream} will be removed from it:`,
    ...lines,
    "",
    "Anyone who already pulled them will have to repair their copy.",
    "If someone pushed after your last fetch, the push is refused and nothing is overwritten.",
  ].join("\n");
}

// ---- "Merge or rebase?" dialog ----

const choice = ref<{ ahead: number; behind: number } | null>(null);
const chosenMode = ref<PullMode>("merge");
// The dialog is awaited like a function: chooseMode() returns a Promise and
// keeps its `resolve` here until the user clicks a button.
let answer: ((mode: PullMode | null) => void) | null = null;

function chooseMode(ahead: number, behind: number, preselected: PullMode): Promise<PullMode | null> {
  chosenMode.value = preselected;
  choice.value = { ahead, behind };
  return new Promise((resolve) => {
    answer = resolve;
  });
}

function closeChoice(mode: PullMode | null) {
  choice.value = null;
  answer?.(mode);
  answer = null;
}
</script>

<template>
  <span class="sync">
    <template v-if="!running">
      <button class="tool" title="Download new commits from all remotes (files stay unchanged)" @click="fetchNow"><AppIcon name="fetch" />Fetch</button>
      <button class="tool" :disabled="busy || pullBlocker !== null" :title="pullBlocker ?? 'Fetch, then bring the new commits into this branch'" @click="pull">
        <AppIcon name="pull" />Pull
      </button>
      <button class="tool" :disabled="pushBlocker !== null" :title="pushBlocker ?? 'Upload this branch\'s new commits to the server'" @click="push">
        <AppIcon name="push" />Push
      </button>
      <button class="tool more" title="More push options (Force Push…)" @click="pushMenu">▾</button>
      <!-- After the buttons, so they don't move when a note appears. -->
      <span v-if="note" class="muted">{{ note }}</span>
    </template>
    <template v-else>
      <span class="progress muted" :title="progress">{{ progress }}</span>
      <button v-if="opId !== null" :disabled="cancelling" @click="cancel">
        {{ cancelling ? "Cancelling…" : "Cancel" }}
      </button>
    </template>
  </span>

  <div v-if="choice" class="backdrop" @mousedown.self="closeChoice(null)">
    <form class="dialog pull-dialog" @submit.prevent="closeChoice(chosenMode)" @keydown.esc="closeChoice(null)">
      <h2>Pull</h2>
      <p>
        This branch has {{ choice.ahead }} local commit(s) that are not on the server, and the server has
        {{ choice.behind }} new commit(s). How should they be combined?
      </p>
      <label class="option">
        <input v-model="chosenMode" type="radio" value="rebase" />
        <span><strong>Rebase</strong>: put your local commits on top of the new ones. History stays a straight line.</span>
      </label>
      <label class="option">
        <input v-model="chosenMode" type="radio" value="merge" />
        <span><strong>Merge</strong>: keep both lines and join them with a merge commit.</span>
      </label>
      <div class="buttons">
        <button type="button" @click="closeChoice(null)">Cancel</button>
        <button type="submit" class="primary">Pull</button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.sync {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
}

.progress {
  max-width: 280px;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

/* The ▾ next to Push. */
/* The ▾ next to Push: a narrow toolbar button of the same height. */
button.tool.more {
  min-width: 0;
  align-self: stretch;
  margin-left: -8px;
  padding: 0 3px;
  color: var(--text-muted);
}

.muted {
  color: var(--text-muted);
}

.pull-dialog {
  width: 440px;
}

.option {
  display: flex;
  gap: 8px;
  align-items: flex-start;
  margin: 8px 0;
}

.option input {
  margin-top: 3px;
}
</style>
