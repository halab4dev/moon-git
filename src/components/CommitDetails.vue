<script setup lang="ts">
import { computed, ref, watch } from "vue";
import DiffView from "./DiffView.vue";
import { getCommitDetails, getCommitFileDiff } from "../api/log";
import { toAppError } from "../api/errors";
import type { AppError } from "../types/error";
import type { FileDiff } from "../types/diff";
import type { CommitDetails } from "../types/log";
import type { ChangeKind, FileChange } from "../types/status";

// Lower half of the History tab: the clicked commit's message and people on
// the left, its changed files below that, and the diff of one file on the right.
const props = defineProps<{
  repoId: string;
  /** null = no commit chosen. */
  commitId: string | null;
}>();

const emit = defineEmits<{
  /** A parent id was clicked. */
  selectCommit: [id: string];
}>();

const details = ref<CommitDetails | null>(null);
const loading = ref(false);
const error = ref<AppError | null>(null);
const selectedPath = ref<string | null>(null);

const subject = computed(() => details.value?.message.split("\n")[0] ?? "");
const body = computed(() => details.value?.message.split("\n").slice(1).join("\n").trim() ?? "");
// Only worth showing when it differs (rebase, cherry-pick, applied patch).
const committerDiffers = computed(() => {
  const d = details.value;
  return (
    d !== null &&
    (d.committerName !== d.authorName || d.committerEmail !== d.authorEmail || d.committerTime !== d.authorTime)
  );
});
const selectedFile = computed(() => details.value?.files.find((f) => f.path === selectedPath.value) ?? null);

// Same "drop late answers" idea as in DiffPanel: clicking quickly through
// commits must never show the details of an earlier one.
let detailsRequest = 0;

watch(
  () => props.commitId,
  async (id) => {
    const request = ++detailsRequest;
    if (id === null) {
      details.value = null;
      error.value = null;
      return;
    }
    loading.value = true;
    try {
      const result = await getCommitDetails(props.repoId, id);
      if (request === detailsRequest) {
        details.value = result;
        error.value = null;
        // Start with the first file, so the diff shows something right away.
        selectedPath.value = result.files[0]?.path ?? null;
      }
    } catch (err) {
      if (request === detailsRequest) {
        details.value = null;
        error.value = toAppError(err);
      }
    } finally {
      if (request === detailsRequest) {
        loading.value = false;
      }
    }
  },
  { immediate: true },
);

// ---- diff of the selected file ----

const diff = ref<FileDiff | null>(null);
const diffLoading = ref(false);
const diffError = ref<AppError | null>(null);
let diffRequest = 0;

watch([details, selectedFile], async ([commit, file]) => {
  const request = ++diffRequest;
  diff.value = null;
  diffError.value = null;
  if (commit === null || file === null) {
    return;
  }
  diffLoading.value = true;
  try {
    // Merges are compared with their first parent, like the file list.
    const parent = commit.parents[0] ?? null;
    const result = await getCommitFileDiff(props.repoId, commit.id, parent, file.path, file.origPath);
    if (request === diffRequest) {
      diff.value = result;
    }
  } catch (err) {
    if (request === diffRequest) {
      diffError.value = toAppError(err);
    }
  } finally {
    if (request === diffRequest) {
      diffLoading.value = false;
    }
  }
});

// ↑/↓ in the file list.
function moveFile(step: number) {
  const files = details.value?.files ?? [];
  if (files.length === 0) {
    return;
  }
  const current = files.findIndex((f) => f.path === selectedPath.value);
  const next = Math.min(files.length - 1, Math.max(0, current + step));
  selectedPath.value = files[next].path;
  document.querySelector(`li[data-path="${CSS.escape(files[next].path)}"]`)?.scrollIntoView({ block: "nearest" });
}

// ---- formatting ----

const KIND_LETTER: Partial<Record<ChangeKind, string>> = {
  added: "A",
  modified: "M",
  deleted: "D",
  renamed: "R",
  copied: "C",
  typeChanged: "T",
};

const dateFormat = new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" });

function formatTime(seconds: number): string {
  return dateFormat.format(new Date(seconds * 1000));
}

function fileName(file: FileChange): string {
  return file.path.split("/").pop() ?? file.path;
}

function folderOf(file: FileChange): string {
  const slash = file.path.lastIndexOf("/");
  return slash === -1 ? "" : file.path.slice(0, slash);
}
</script>

<template>
  <div class="commit-details">
    <p v-if="commitId === null" class="muted placeholder">Select a commit to see its details.</p>
    <p v-else-if="error" class="err placeholder">{{ error.message }}</p>
    <p v-else-if="!details && loading" class="muted placeholder">Loading…</p>

    <template v-else-if="details">
      <div class="side">
        <div class="info">
          <h3 class="subject">{{ subject }}</h3>
          <pre v-if="body" class="body">{{ body }}</pre>
          <dl>
            <dt>Commit</dt>
            <dd class="hash selectable">{{ details.id }}</dd>
            <template v-if="details.parents.length > 0">
              <dt>{{ details.parents.length > 1 ? "Parents" : "Parent" }}</dt>
              <dd>
                <button
                  v-for="parent in details.parents"
                  :key="parent"
                  class="link hash"
                  title="Show this commit"
                  @click="emit('selectCommit', parent)"
                >
                  {{ parent.slice(0, 8) }}
                </button>
              </dd>
            </template>
            <dt>Author</dt>
            <dd class="selectable">
              {{ details.authorName }} &lt;{{ details.authorEmail }}&gt;
              <span class="muted">· {{ formatTime(details.authorTime) }}</span>
            </dd>
            <template v-if="committerDiffers">
              <dt>Committer</dt>
              <dd class="selectable">
                {{ details.committerName }} &lt;{{ details.committerEmail }}&gt;
                <span class="muted">· {{ formatTime(details.committerTime) }}</span>
              </dd>
            </template>
          </dl>
        </div>

        <p class="files-title muted">
          {{ details.files.length }} changed file(s){{ details.parents.length > 1 ? " (compared with the first parent)" : "" }}
        </p>
        <ul class="files" tabindex="0" @keydown.up.prevent="moveFile(-1)" @keydown.down.prevent="moveFile(1)">
          <li
            v-for="file in details.files"
            :key="file.path"
            :data-path="file.path"
            :class="{ selected: file.path === selectedPath }"
            :title="file.origPath ? `${file.origPath} → ${file.path}` : file.path"
            @click="selectedPath = file.path"
          >
            <span class="badge" :class="file.kind">{{ KIND_LETTER[file.kind] ?? "?" }}</span>
            <span class="name">{{ fileName(file) }}</span>
            <span class="folder">{{ folderOf(file) }}</span>
          </li>
        </ul>
      </div>

      <DiffView
        class="diff"
        :path="selectedFile?.path ?? null"
        :diff="diff"
        :loading="diffLoading"
        :error="diffError"
        :placeholder="details.files.length === 0 ? 'This commit changes no files.' : 'Select a file to see its changes.'"
      />
    </template>
  </div>
</template>

<style scoped>
.commit-details {
  display: flex;
  height: 100%;
  min-height: 0;
  border-top: 1px solid var(--border);
}

.placeholder {
  margin: 12px 0;
}

/* Left column: message, people, file list. */
.side {
  display: flex;
  flex-direction: column;
  width: min(40%, 420px);
  flex-shrink: 0;
  min-height: 0;
  padding-right: 10px;
  border-right: 1px solid var(--border);
}

.info {
  flex-shrink: 0;
  max-height: 50%;
  overflow-y: auto;
  padding: 6px 0;
}

.subject {
  margin: 0 0 6px;
  font-size: 14px;
  user-select: text;
}

.body {
  margin: 0 0 8px;
  white-space: pre-wrap;
  word-break: break-word;
  font-family: inherit;
  user-select: text;
}

dl {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 2px 10px;
  margin: 0;
}

dt {
  color: var(--text-muted);
}

dd {
  margin: 0;
  min-width: 0;
  overflow-wrap: anywhere;
}

.selectable {
  user-select: text;
}

.hash {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
}

button.link {
  border: none;
  background: none;
  padding: 0;
  margin-right: 8px;
  color: var(--accent);
  cursor: pointer;
}

.files-title {
  margin: 6px 0 2px;
  font-size: 12px;
}

.files {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  margin: 0;
  padding: 0;
  list-style: none;
}

.files:focus {
  outline: none;
}

.files li {
  display: flex;
  gap: 6px;
  align-items: baseline;
  padding: 2px 4px;
  white-space: nowrap;
  cursor: default;
  user-select: none;
}

.files li:hover {
  background: var(--bg-sidebar);
}

.files li.selected {
  background: var(--accent);
  color: #fff;
}

.files li.selected .folder,
.files li.selected .badge {
  color: #fff;
}

.name {
  flex-shrink: 0;
}

.folder {
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--text-muted);
}

.badge {
  width: 14px;
  flex-shrink: 0;
  text-align: center;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-weight: 700;
}

.badge.added {
  color: var(--success);
}

.badge.modified,
.badge.typeChanged {
  color: var(--warning);
}

.badge.deleted {
  color: var(--danger);
}

.badge.renamed,
.badge.copied {
  color: var(--accent);
}

.diff {
  flex: 1;
  min-width: 0;
  padding-left: 10px;
}

.muted {
  color: var(--text-muted);
}
</style>
