import { defineStore } from "pinia";
import { ref } from "vue";

// App-wide dialogs live here so any code (e.g. a native menu action) can open
// them. The dialog components themselves are mounted once, in App.vue.

export interface NameRequest {
  title: string;
  /** Text of the confirm button, e.g. "Create" or "Rename". */
  confirmLabel: string;
  initial: string;
  /** Returns an error message to show in the dialog, or null to close it. */
  submit: (name: string) => Promise<string | null>;
}

export interface CloneRequest {
  /** Group the cloned repo goes into; null = top level. */
  groupId: string | null;
}

export const useDialogStore = defineStore("dialogs", () => {
  const nameRequest = ref<NameRequest | null>(null);
  const cloneRequest = ref<CloneRequest | null>(null);

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

  return { nameRequest, cloneRequest, askName, closeName, openClone, closeClone };
});
