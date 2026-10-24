<script setup lang="ts">
import { defineAsyncComponent, onMounted, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ask } from "@tauri-apps/plugin-dialog";
import { getConflictVersions, saveConflictResult } from "../api/stage";
import { toAppError } from "../api/errors";
import type { AppError } from "../types/error";
import type { ConflictVersions } from "../types/status";

// Root of a conflict solver window (see windows.ts), opened by
// double-clicking a conflicted file. The file and the names of both sides
// come from the URL: index.html?view=conflict&repoId=…&path=…&ours=…&theirs=…
const params = new URLSearchParams(window.location.search);
const repoId = params.get("repoId") ?? "";
const path = params.get("path") ?? "";
const oursLabel = params.get("ours") ?? "ours";
const theirsLabel = params.get("theirs") ?? "theirs";

// Monaco is large: loaded only when the solver is shown (as in DiffView).
const ConflictSolver = defineAsyncComponent(() => import("../components/ConflictSolver.vue"));
const solver = ref<{ markSaved: (content: string) => void } | null>(null);

const versions = ref<ConflictVersions | null>(null);
const error = ref<AppError | null>(null);
const saving = ref(false);
const dirty = ref(false);

onMounted(async () => {
  try {
    versions.value = await getConflictVersions(repoId, path);
  } catch (err) {
    error.value = toAppError(err);
  }
});

// Closing with unsaved edits asks first. Tauri waits for this handler and
// closes the window unless preventDefault() was called.
void getCurrentWindow().onCloseRequested(async (event) => {
  if (!dirty.value) {
    return;
  }
  const close = await ask("The result has unsaved changes. Close without saving?", {
    title: "Unsaved Changes",
    kind: "warning",
    okLabel: "Close Without Saving",
    cancelLabel: "Keep Editing",
  });
  if (!close) {
    event.preventDefault();
  }
});

async function onSave(content: string, markResolved: boolean) {
  saving.value = true;
  error.value = null;
  try {
    await write(content, markResolved, false);
  } catch (err) {
    const first = toAppError(err);
    // Markers left: the file is saved already; ask before staging it anyway.
    if (first.kind !== "conflictMarkers") {
      error.value = first;
      return;
    }
    solver.value?.markSaved(content);
    const anyway = await ask(`${first.message}\n\nMark as resolved anyway?`, {
      title: "Conflict Markers Left",
      kind: "warning",
      okLabel: "Mark Resolved",
      cancelLabel: "Keep Editing",
    });
    if (anyway) {
      try {
        await write(content, true, true);
      } catch (again) {
        error.value = toAppError(again);
      }
    }
  } finally {
    saving.value = false;
  }
}

async function write(content: string, markResolved: boolean, force: boolean) {
  await saveConflictResult(repoId, path, content, markResolved, force);
  solver.value?.markSaved(content);
  if (markResolved) {
    // Done with this file. The main window reloads when it gets the focus back.
    await getCurrentWindow().close();
  }
}
</script>

<template>
  <div class="conflict-window">
    <header>
      <strong>{{ path.slice(path.lastIndexOf("/") + 1) }}</strong>
      <span class="muted">{{ path }}</span>
    </header>
    <p v-if="error" class="err message">{{ error.message }}</p>

    <template v-if="versions">
      <p v-if="versions.tooLarge" class="muted message">This file is too large to resolve here.</p>
      <p v-else-if="versions.binary" class="muted message">
        This file is not text (or not UTF-8), so it can't be merged line by line. In the main window, right-click it and
        choose Take Ours or Take Theirs.
      </p>
      <p v-else-if="versions.missingSide" class="muted message">
        One side deleted this file and the other changed it, so there is nothing to merge line by line. In the main
        window, right-click it and choose Take Ours or Take Theirs (taking the side that deleted it deletes the file).
      </p>
      <div v-else class="body">
        <ConflictSolver
          ref="solver"
          :path="path"
          :base="versions.base"
          :ours="versions.ours ?? ''"
          :theirs="versions.theirs ?? ''"
          :merged="versions.merged ?? ''"
          :ours-label="oursLabel"
          :theirs-label="theirsLabel"
          :saving="saving"
          @save="onSave"
          @dirty="(value) => (dirty = value)"
        />
      </div>
    </template>
    <p v-else-if="!error" class="muted message">Loading…</p>
  </div>
</template>

<style scoped>
.conflict-window {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 8px 12px;
}

header {
  display: flex;
  align-items: baseline;
  gap: 10px;
  padding-bottom: 6px;
  white-space: nowrap;
  overflow: hidden;
}

.muted {
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
}

.message {
  margin: 12px 0;
  white-space: pre-wrap;
}

.body {
  flex: 1;
  min-height: 0;
}
</style>
