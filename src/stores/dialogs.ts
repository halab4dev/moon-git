import { acceptHMRUpdate, defineStore } from "pinia";
import { ref } from "vue";
import type { MergeMode, MergePreview } from "../types/merge";

// App-wide dialogs live here so any code (e.g. a native menu action) can open
// them. The dialog components themselves are mounted once, in App.vue.

export interface NameRequest {
  title: string;
  /** Text of the confirm button, e.g. "Create" or "Rename". */
  confirmLabel: string;
  initial: string;
  /** Hint in the empty input; default "Name". */
  placeholder?: string;
  /** The input may stay empty (e.g. an optional stash message). */
  allowEmpty?: boolean;
  /** Optional checkbox under the input, e.g. "Check out the new branch". */
  option?: { label: string; initial: boolean };
  /**
   * Returns an error message to show in the dialog, or null to close it.
   * `optionChecked`: state of the checkbox (false when there is none).
   */
  submit: (name: string, optionChecked: boolean) => Promise<string | null>;
}

export interface MergeRequest {
  preview: MergePreview;
  /** Returns an error message to show in the dialog, or null to close it. */
  submit: (mode: MergeMode) => Promise<string | null>;
}

export interface CloneRequest {
  /** Group the cloned repo goes into; null = top level. */
  groupId: string | null;
}

export const useDialogStore = defineStore("dialogs", () => {
  const nameRequest = ref<NameRequest | null>(null);
  const cloneRequest = ref<CloneRequest | null>(null);
  const mergeRequest = ref<MergeRequest | null>(null);

  function askName(request: NameRequest) {
    nameRequest.value = request;
  }

  function closeName() {
    nameRequest.value = null;
  }

  function openClone(groupId: string | null) {
    cloneRequest.value = { groupId };
  }

  function closeClone() {
    cloneRequest.value = null;
  }

  function askMerge(request: MergeRequest) {
    mergeRequest.value = request;
  }

  function closeMerge() {
    mergeRequest.value = null;
  }

  return {
    nameRequest,
    cloneRequest,
    mergeRequest,
    askName,
    closeName,
    openClone,
    closeClone,
    askMerge,
    closeMerge,
  };
});

// During development, let hot reload replace this store's code too. Without
// it, the store created before an edit keeps its old functions until a full
// reload (⌘R), and newly added ones are missing.
if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useDialogStore, import.meta.hot));
}
