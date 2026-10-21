import { defineStore } from "pinia";
import { computed, ref } from "vue";
import * as repoApi from "../api/repo";
import { toAppError } from "../api/errors";
import type { RepoEntry } from "../types/repo";
import type { AppError } from "../types/error";

// Setup-style store: state is ref(), derived values are computed(),
// actions are plain functions. Rust owns the saved list; this is the UI copy.
export const useRepoStore = defineStore("repos", () => {
  const repos = ref<RepoEntry[]>([]);
  const selectedId = ref<string | null>(null);
  const error = ref<AppError | null>(null);

  const sortedRepos = computed(() =>
    [...repos.value].sort((a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: "base" })),
  );

  const selected = computed(() => repos.value.find((r) => r.id === selectedId.value) ?? null);

  async function load() {
    await run(async () => {
      repos.value = await repoApi.listRepos();
    });
  }

  async function add(path: string) {
    await run(async () => {
      const entry = await repoApi.addRepo(path);
      repos.value.push(entry);
      selectedId.value = entry.id;
    });
  }

  async function remove(id: string) {
    await run(async () => {
      await repoApi.removeRepo(id);
      repos.value = repos.value.filter((r) => r.id !== id);
      if (selectedId.value === id) {
        selectedId.value = null;
      }
    });
  }

  function select(id: string) {
    selectedId.value = id;
  }

  function clearError() {
    error.value = null;
  }

  // Keeps the last failure in `error` for the UI to show, instead of throwing.
  async function run(action: () => Promise<void>) {
    error.value = null;
    try {
      await action();
    } catch (err) {
      error.value = toAppError(err);
    }
  }

  return { repos, sortedRepos, selectedId, selected, error, load, add, remove, select, clearError };
});
