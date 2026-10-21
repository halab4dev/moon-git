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

export const useDialogStore = defineStore("dialogs", () => {
  const nameRequest = ref<NameRequest | null>(null);

  function askName(request: NameRequest) {
    nameRequest.value = request;
  }

  function closeName() {
    nameRequest.value = null;
  }

  return { nameRequest, askName, closeName };
});
