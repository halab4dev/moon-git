import { acceptHMRUpdate, defineStore } from "pinia";
import { computed, ref } from "vue";
import * as repoApi from "../api/repo";
import * as groupApi from "../api/group";
import * as cloneApi from "../api/clone";
import { applyGroupIdentity, applyIdentityToRepos, type GroupApplyResult } from "../api/identity";
import { applyAccessToRepos, applyGroupAccess } from "../api/access";
import { toAppError } from "../api/errors";
import { buildTree, listGroupOptions, rangeBetween, visibleRepoOrder } from "./repoTree";
import type { RepoEntry, RepoGroup, RepoList } from "../types/repo";
import type { AppError } from "../types/error";

// Setup-style store: state is ref(), derived values are computed(),
// actions are plain functions. Rust owns the saved list; this is the UI copy.
//
// Actions throw an AppError on failure. Callers decide where to show it:
// `attempt()` puts it in the sidebar error box, the name dialog shows it inline.
export const useRepoStore = defineStore("repos", () => {
  const groups = ref<RepoGroup[]>([]);
  const repos = ref<RepoEntry[]>([]);
  const selectedId = ref<string | null>(null);
  // Several repositories picked with ⌘-click / ⇧-click (step 35b), for
  // dragging them together and actions on all of them. The opened one
  // (selectedId) is in it after a plain click.
  const picked = ref(new Set<string>());
  // Where a ⇧-click range starts: the last repository clicked without ⇧.
  let anchorId: string | null = null;
  // Collapsed groups. Not saved: every group starts expanded.
  const collapsed = ref(new Set<string>());
  const error = ref<AppError | null>(null);

  const tree = computed(() => buildTree(groups.value, repos.value));
  const groupOptions = computed(() => listGroupOptions(tree.value));
  const selected = computed(() => repos.value.find((r) => r.id === selectedId.value) ?? null);
  const isEmpty = computed(() => groups.value.length === 0 && repos.value.length === 0);
  const pickedRepos = computed(() => repos.value.filter((r) => picked.value.has(r.id)));

  function applyList(list: RepoList) {
    groups.value = list.groups;
    repos.value = list.repos;
    if (selected.value === null) {
      selectedId.value = null;
    }
    // Forget picked repositories that were removed.
    const ids = new Set(list.repos.map((r) => r.id));
    picked.value = new Set([...picked.value].filter((id) => ids.has(id)));
  }

  async function load() {
    applyList(await call(repoApi.listRepos()));
  }

  async function addRepo(path: string, groupId: string | null) {
    insertRepo(await call(repoApi.addRepo(path, groupId)));
  }

  async function cloneRepo(
    opId: string,
    url: string,
    parentDir: string,
    folderName: string,
    groupId: string | null,
  ) {
    insertRepo(await call(cloneApi.cloneRepo(opId, url, parentDir, folderName, groupId)));
  }

  // Show a just-added repo: put it in the list, select it, expand its group.
  function insertRepo(entry: RepoEntry) {
    repos.value.push(entry);
    selectedId.value = entry.id;
    revealGroup(entry.groupId);
  }

  async function removeRepo(id: string) {
    applyList(await call(repoApi.removeRepo(id)));
  }

  async function moveRepo(repoId: string, groupId: string | null) {
    applyList(await call(repoApi.moveRepo(repoId, groupId)));
    revealGroup(groupId);
  }

  async function createGroup(name: string, parentId: string | null) {
    applyList(await call(groupApi.createGroup(name, parentId)));
    revealGroup(parentId);
  }

  async function renameGroup(id: string, name: string) {
    applyList(await call(groupApi.renameGroup(id, name)));
  }

  /** Step 14b: saves the group's identity and writes it into its repositories. */
  async function setGroupIdentity(id: string, name: string, email: string, all: boolean): Promise<GroupApplyResult> {
    const result = await call(applyGroupIdentity(id, name, email, all));
    applyList(result.list);
    return result;
  }

  /** Step 15b: writes login settings into the repositories of a group. */
  async function setGroupAccess(
    id: string,
    sshKey: string | null,
    systemCredentials: boolean,
    all: boolean,
  ): Promise<GroupApplyResult> {
    const result = await call(applyGroupAccess(id, sshKey, systemCredentials, all));
    applyList(result.list);
    return result;
  }

  /** Step 35c: identity for the picked repositories. */
  async function setReposIdentity(repoIds: string[], name: string, email: string, all: boolean): Promise<GroupApplyResult> {
    const result = await call(applyIdentityToRepos(repoIds, name, email, all));
    applyList(result.list);
    return result;
  }

  /** Step 35c: login settings for the picked repositories. */
  async function setReposAccess(
    repoIds: string[],
    sshKey: string | null,
    systemCredentials: boolean,
    all: boolean,
  ): Promise<GroupApplyResult> {
    const result = await call(applyAccessToRepos(repoIds, sshKey, systemCredentials, all));
    applyList(result.list);
    return result;
  }

  async function moveGroup(id: string, parentId: string | null) {
    applyList(await call(groupApi.moveGroup(id, parentId)));
    revealGroup(parentId);
  }

  async function deleteGroup(id: string) {
    applyList(await call(groupApi.deleteGroup(id)));
    collapsed.value.delete(id);
  }

  /** Plain click: open this repository, and pick only it. */
  function select(id: string) {
    selectedId.value = id;
    picked.value = new Set([id]);
    anchorId = id;
  }

  /** ⌘-click: add or remove one repository; the opened one stays open. */
  function togglePicked(id: string) {
    const next = new Set(picked.value);
    if (!next.delete(id)) {
      next.add(id);
    }
    picked.value = next;
    anchorId = id;
  }

  /** ⇧-click: every visible repository from the last clicked one to this one. */
  function pickRange(id: string) {
    const order = visibleRepoOrder(tree.value, isCollapsed);
    picked.value = new Set(rangeBetween(order, anchorId ?? selectedId.value, id));
  }

  function isPicked(id: string): boolean {
    return picked.value.has(id);
  }

  /** Back to just the opened repository (e.g. after acting on several). */
  function clearPicked() {
    picked.value = new Set(selectedId.value === null ? [] : [selectedId.value]);
  }

  function isCollapsed(groupId: string): boolean {
    return collapsed.value.has(groupId);
  }

  function toggleCollapsed(groupId: string) {
    if (collapsed.value.has(groupId)) {
      collapsed.value.delete(groupId);
    } else {
      collapsed.value.add(groupId);
    }
  }

  // Expand a group and all its parents so a newly placed item is visible.
  function revealGroup(groupId: string | null) {
    let current = groupId;
    while (current !== null) {
      collapsed.value.delete(current);
      current = groups.value.find((g) => g.id === current)?.parentId ?? null;
    }
  }

  /** Runs an action and shows any failure in the sidebar error box. */
  async function attempt(action: () => Promise<void>) {
    error.value = null;
    try {
      await action();
    } catch (err) {
      error.value = toAppError(err);
    }
  }

  function clearError() {
    error.value = null;
  }

  return {
    groups,
    repos,
    selectedId,
    error,
    tree,
    groupOptions,
    selected,
    isEmpty,
    load,
    addRepo,
    cloneRepo,
    removeRepo,
    moveRepo,
    createGroup,
    renameGroup,
    setGroupIdentity,
    setGroupAccess,
    setReposIdentity,
    setReposAccess,
    moveGroup,
    deleteGroup,
    select,
    togglePicked,
    pickRange,
    isPicked,
    clearPicked,
    pickedRepos,
    isCollapsed,
    toggleCollapsed,
    attempt,
    revealGroup,
    clearError,
  };
});

// Await an API call and turn whatever it rejects with into an AppError.
async function call<T>(promise: Promise<T>): Promise<T> {
  try {
    return await promise;
  } catch (err) {
    throw toAppError(err);
  }
}

// During development, let hot reload replace this store's code too (as in
// dialogs.ts). Without it, the store created before an edit keeps its old
// functions until a full reload, and newly added ones (moveGroup) are missing.
if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useRepoStore, import.meta.hot));
}
