import { acceptHMRUpdate, defineStore } from "pinia";
import { ref, shallowRef } from "vue";
import type { RepoStatus } from "../types/status";

// Lets the branch panel in the sidebar and the RepoView of the selected repo
// work together, although neither contains the other:
// - RepoView bumps `statusVersion` whenever it shows a new status (commit,
//   checkout, pull…), so the panel reloads its branches;
// - RepoView mirrors its "a git action is running" flag in `busy`;
// - a branch action in the panel (checkout…) hands its new status to
//   RepoView through `incomingStatus`, so the status keeps one owner.
export const useRepoActivity = defineStore("repoActivity", () => {
  const statusVersion = ref(0);
  const busy = ref(false);
  const incomingStatus = shallowRef<RepoStatus | null>(null);

  function statusChanged() {
    statusVersion.value++;
  }

  function deliverStatus(status: RepoStatus) {
    incomingStatus.value = status;
  }

  return { statusVersion, busy, incomingStatus, statusChanged, deliverStatus };
});

// See dialogs.ts: lets hot reload replace this store's code during development.
if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useRepoActivity, import.meta.hot));
}
