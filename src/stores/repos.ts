import { defineStore } from "pinia";
import { computed, ref } from "vue";
import * as repoApi from "../api/repo";
import * as groupApi from "../api/group";
import * as cloneApi from "../api/clone";
import { toAppError } from "../api/errors";
import { buildTree, listGroupOptions } from "./repoTree";
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
  // Collapsed groups. Not saved: every group starts expanded.
  const collapsed = ref(new Set<string>());
  const error = ref<AppError | null>(null);

  const tree = computed(() => buildTree(groups.value, repos.value));
  const groupOptions = computed(() => listGroupOptions(tree.value));
  const selected = computed(() => repos.value.find((r) => r.id === selectedId.value) ?? null);
  const isEmpty = computed(() => groups.value.length === 0 && repos.value.length === 0);

  function applyList(list: RepoList) {
    groups.value = list.groups;
    repos.value = list.repos;
    if (selected.value === null) {
      selectedId.value = null;
    }
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

  async function deleteGroup(id: string) {
    applyList(await call(groupApi.deleteGroup(id)));
    collapsed.value.delete(id);
  }

  function select(id: string) {
    selectedId.value = id;
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
    deleteGroup,
    select,
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
