<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { Menu } from "@tauri-apps/api/menu";
import { message } from "@tauri-apps/plugin-dialog";
import { addRemote, listBranches, setRemoteUrl } from "../api/branch";
import { toAppError } from "../api/errors";
import { displayUrl } from "./remoteUrl";
import { checkOutItem, showBranchItemMenu, type BranchCallbacks, type BranchItem } from "./branchMenus";
import { useDialogStore } from "../stores/dialogs";
import { useRepoActivity } from "../stores/repoActivity";
import { useMenuHandlers } from "../stores/menuCommands";
import type { AppError } from "../types/error";
import type { BranchList, LocalBranch, RemoteGroup } from "../types/branches";

// Lower part of the sidebar (step 25): the branches of the selected repo,
// like SmartGit's Branches view. Local branches first, then the remote
// branches, one group per remote. Double-click checks out; right-click
// opens the branch actions.
const props = defineProps<{
  repoId: string;
}>();

const activity = useRepoActivity();
const branches = ref<BranchList | null>(null);
const error = ref<AppError | null>(null);
// The clicked row (by full ref name), only for highlighting.
const selected = ref<string | null>(null);

let request = 0;

async function load() {
  const current = ++request;
  try {
    const result = await listBranches(props.repoId);
    if (current === request) {
      branches.value = result;
      error.value = null;
    }
  } catch (err) {
    if (current === request) {
      error.value = toAppError(err);
    }
  }
}

onMounted(load);
// RepoView shows a new status after every git action: the branches may have changed.
watch(() => activity.statusVersion, load);

// Results go to RepoView (which owns the status). A failed action (e.g. a
// checkout blocked by uncommitted changes) is shown in a dialog, so it can't
// be missed and doesn't push the list away; the line above the list is only
// for "the list itself could not be loaded".
const callbacks: BranchCallbacks = {
  onStatus: (status) => activity.deliverStatus(status),
  onError: (err) => void message(err.message, { title: "Branch Action Failed", kind: "error" }),
};

function localItem(branch: LocalBranch): BranchItem {
  return { fullName: branch.fullName, name: branch.name, kind: "branch", current: branch.current };
}

function remoteItem(remote: string, name: string, fullName: string): BranchItem {
  return { fullName, name: `${remote}/${name}`, kind: "remote", current: false };
}

function open(item: BranchItem) {
  selected.value = item.fullName;
  if (!activity.busy) {
    checkOutItem(props.repoId, item, callbacks);
  }
}

function menu(item: BranchItem) {
  selected.value = item.fullName;
  if (!activity.busy) {
    void showBranchItemMenu(props.repoId, item, callbacks);
  }
}

// ---- adding a remote, changing its URL (step 26) ----

const dialogs = useDialogStore();

function askAddRemote() {
  const names = branches.value?.remotes.map((group) => group.name) ?? [];
  dialogs.askRemote({
    existing: null,
    // The usual name for the first one; a second one needs its own name.
    initialName: names.length === 0 ? "origin" : "",
    submit: async (name, url) => {
      try {
        await addRemote(props.repoId, name, url);
      } catch (err) {
        return toAppError(err).message;
      }
      await load();
      return null;
    },
  });
}

function askEditUrl(name: string, url: string) {
  dialogs.askRemote({
    existing: { name, url },
    initialName: name,
    submit: async (_name, newUrl) => {
      try {
        await setRemoteUrl(props.repoId, name, newUrl);
      } catch (err) {
        return toAppError(err).message;
      }
      await load();
      return null;
    },
  });
}

/** Right-click on a remote's group: change its URL (not for leftovers of a removed remote). */
async function remoteMenu(group: RemoteGroup) {
  const url = group.url;
  if (url === null) {
    return;
  }
  const menu = await Menu.new({
    items: [
      { text: `Edit URL of ${group.name}…`, action: () => askEditUrl(group.name, url) },
      { item: "Separator" },
      { text: "Add Remote…", action: askAddRemote },
    ],
  });
  await menu.popup();
}

// Remote → Add Remote… in the menu bar.
useMenuHandlers({
  "remote.addRemote": { run: askAddRemote, enabled: () => branches.value !== null && !activity.busy },
});

async function remotesMenu() {
  const menu = await Menu.new({ items: [{ text: "Add Remote…", action: askAddRemote }] });
  await menu.popup();
}

// ---- sorting by name, A→Z or Z→A, remembered for all repos ----

type SortOrder = "asc" | "desc";
const SORT_KEY = "moongit.branches.sort";
const sortOrder = ref<SortOrder>(readSortOrder());

function readSortOrder(): SortOrder {
  try {
    return localStorage.getItem(SORT_KEY) === "desc" ? "desc" : "asc";
  } catch {
    return "asc";
  }
}

function toggleSort() {
  sortOrder.value = sortOrder.value === "asc" ? "desc" : "asc";
  try {
    localStorage.setItem(SORT_KEY, sortOrder.value);
  } catch {
    // Not remembered; fine.
  }
}

// Like the file table (step 6): "feature-2" before "feature-10", and upper /
// lower case sorted together.
const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });

function byName<T extends { name: string }>(items: T[]): T[] {
  const sign = sortOrder.value === "asc" ? 1 : -1;
  return [...items].sort((a, b) => sign * collator.compare(a.name, b.name));
}

// The branches as shown: sorted inside each group. The remote groups
// themselves stay in name order (origin, upstream…).
const shown = computed(() => {
  const list = branches.value;
  if (list === null) {
    return null;
  }
  return {
    local: byName(list.local),
    remotes: list.remotes.map((group) => ({ ...group, branches: byName(group.branches) })),
  };
});

// ---- collapsed groups, remembered for all repos ----

const COLLAPSED_KEY = "moongit.branches.collapsed";
const collapsed = ref(new Set<string>(readCollapsed()));

function readCollapsed(): string[] {
  try {
    const saved = JSON.parse(localStorage.getItem(COLLAPSED_KEY) ?? "[]");
    return Array.isArray(saved) ? saved : [];
  } catch {
    return [];
  }
}

function toggle(key: string) {
  const next = new Set(collapsed.value);
  if (!next.delete(key)) {
    next.add(key);
  }
  collapsed.value = next;
  try {
    localStorage.setItem(COLLAPSED_KEY, JSON.stringify([...next]));
  } catch {
    // Not remembered; fine.
  }
}

// "↑1 ↓2", "gone", or "" (in sync / no upstream).
function trackText(branch: LocalBranch): string {
  if (branch.upstreamGone) {
    return "gone";
  }
  return [branch.ahead > 0 ? `↑${branch.ahead}` : "", branch.behind > 0 ? `↓${branch.behind}` : ""].filter(Boolean).join(" ");
}

function localTitle(branch: LocalBranch): string {
  if (branch.upstream === null) {
    return `${branch.name} (no upstream)`;
  }
  return branch.upstreamGone ? `${branch.name} → ${branch.upstream} (deleted on the server)` : `${branch.name} → ${branch.upstream}`;
}
</script>

<template>
  <div class="branch-panel">
    <header class="panel-header">
      <span>Branches</span>
      <span class="header-buttons">
        <button
          class="icon-btn sort"
          :title="sortOrder === 'asc' ? 'Sorted by name A→Z. Click for Z→A' : 'Sorted by name Z→A. Click for A→Z'"
          @click="toggleSort"
        >
          {{ sortOrder === "asc" ? "A→Z" : "Z→A" }}
        </button>
        <button class="icon-btn" title="Reload the branches" @click="load">↻</button>
      </span>
    </header>
    <!-- Above the list, never instead of it: limited height, can be closed. -->
    <div v-if="error" class="error">
      <span>{{ error.message }}</span>
      <button class="icon-btn" title="Dismiss" @click="error = null">×</button>
    </div>

    <div v-if="shown" class="panel-body">
      <ul class="tree">
        <li>
          <div class="group" @click="toggle('local')">
            <span class="chevron">{{ collapsed.has("local") ? "▸" : "▾" }}</span>
            Local Branches
            <span class="count">{{ shown.local.length }}</span>
          </div>
          <ul v-if="!collapsed.has('local')">
            <li
              v-for="branch in shown.local"
              :key="branch.fullName"
              class="branch"
              :class="{ current: branch.current, selected: selected === branch.fullName }"
              :title="localTitle(branch)"
              @click="selected = branch.fullName"
              @dblclick="open(localItem(branch))"
              @contextmenu.prevent="menu(localItem(branch))"
            >
              <span class="mark">{{ branch.current ? "✓" : "" }}</span>
              <span class="name">{{ branch.name }}</span>
              <span class="track" :class="{ gone: branch.upstreamGone }">{{ trackText(branch) }}</span>
            </li>
            <li v-if="shown.local.length === 0" class="muted branch">No local branches yet.</li>
          </ul>
        </li>

        <li>
          <div class="group" @click="toggle('remotes')" @contextmenu.prevent="remotesMenu">
            <span class="chevron">{{ collapsed.has("remotes") ? "▸" : "▾" }}</span>
            Remotes
            <span class="count">{{ shown.remotes.length }}</span>
            <button class="icon-btn add" title="Add Remote…" @click.stop="askAddRemote">+</button>
          </div>
          <ul v-if="!collapsed.has('remotes')">
            <li v-for="remote in shown.remotes" :key="remote.name">
              <div
                class="group nested"
                :title="remote.url === null ? remote.name : `${remote.name}: ${displayUrl(remote.url)}`"
                @click="toggle(`remote:${remote.name}`)"
                @contextmenu.prevent="remoteMenu(remote)"
              >
                <span class="chevron">{{ collapsed.has(`remote:${remote.name}`) ? "▸" : "▾" }}</span>
                <span class="remote-name">{{ remote.name }}</span>
                <span v-if="remote.url !== null" class="url">{{ displayUrl(remote.url) }}</span>
                <span class="count">{{ remote.branches.length }}</span>
              </div>
              <ul v-if="!collapsed.has(`remote:${remote.name}`)">
                <li
                  v-for="branch in remote.branches"
                  :key="branch.fullName"
                  class="branch nested"
                  :class="{ selected: selected === branch.fullName }"
                  :title="`${remote.name}/${branch.name}`"
                  @click="selected = branch.fullName"
                  @dblclick="open(remoteItem(remote.name, branch.name, branch.fullName))"
                  @contextmenu.prevent="menu(remoteItem(remote.name, branch.name, branch.fullName))"
                >
                  <span class="mark"></span>
                  <span class="name">{{ branch.name }}</span>
                </li>
                <li v-if="remote.branches.length === 0" class="muted branch nested">No branches (not fetched yet?).</li>
              </ul>
            </li>
            <li v-if="shown.remotes.length === 0" class="muted branch">
              No remotes. <a href="#" @click.prevent="askAddRemote">Add Remote…</a>
            </li>
          </ul>
        </li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.branch-panel {
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100%;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px 6px 12px;
  font-weight: 600;
  text-transform: uppercase;
  font-size: 11px;
  letter-spacing: 0.05em;
  color: var(--text-muted);
  border-top: 1px solid var(--border);
  border-bottom: 1px solid var(--border);
}

.header-buttons {
  display: flex;
  gap: 2px;
}

.icon-btn.sort {
  width: auto;
  padding: 0 4px;
  font-size: 10px;
  text-transform: none;
}

.error {
  display: flex;
  align-items: flex-start;
  gap: 4px;
  flex-shrink: 0;
  max-height: 30%;
  overflow-y: auto;
  margin: 6px 8px;
  padding: 4px 6px;
  border: 1px solid var(--danger);
  border-radius: 4px;
  color: var(--danger);
  word-break: break-word;
}

.error span {
  flex: 1;
}

.panel-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 4px 0;
}

ul {
  margin: 0;
  padding: 0;
  list-style: none;
}

.group {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  font-weight: 600;
  cursor: default;
  user-select: none;
}

.group.nested {
  padding-left: 22px;
  font-weight: 500;
}

.icon-btn.add {
  width: 16px;
  height: 16px;
  line-height: 14px;
  font-weight: 400;
}

.muted a {
  color: var(--accent);
}

.remote-name {
  flex-shrink: 0;
}

/* Muted, and cut with "…" when the sidebar is narrow (full URL in the tooltip). */
.url {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-muted);
  font-weight: 400;
  font-size: 11px;
}

.chevron {
  width: 12px;
  color: var(--text-muted);
}

.count {
  margin-left: auto;
  color: var(--text-muted);
  font-weight: 400;
  font-size: 11px;
}

.branch {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px 2px 22px;
  white-space: nowrap;
  cursor: default;
  user-select: none;
}

.branch.nested {
  padding-left: 36px;
}

.branch:hover {
  background: rgba(127, 127, 127, 0.12);
}

.branch.selected {
  background: var(--accent);
  color: #fff;
}

.branch.selected .track,
.branch.selected .mark {
  color: #fff;
}

.branch.current .name {
  font-weight: 700;
}

.mark {
  width: 12px;
  flex-shrink: 0;
  color: var(--success);
}

.name {
  overflow: hidden;
  text-overflow: ellipsis;
}

.track {
  margin-left: auto;
  padding-left: 6px;
  color: var(--text-muted);
  font-size: 11px;
}

.track.gone {
  color: var(--warning);
}

.muted {
  color: var(--text-muted);
}
</style>
