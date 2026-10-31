import { acceptHMRUpdate, defineStore } from "pinia";
import { ref } from "vue";
import type { MergeMode, MergePreview } from "../types/merge";
import type { ResetMode, ResetPreview } from "../types/reset";
import type { CherryPickMode } from "../types/cherryPick";

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

export interface ResetRequest {
  preview: ResetPreview;
  /** Returns an error message to show in the dialog, or null to close it. */
  submit: (mode: ResetMode) => Promise<string | null>;
}

export interface CherryPickRequest {
  commitId: string;
  subject: string;
  /** Picked relative to its first parent. */
  isMerge: boolean;
  /** Returns an error message to show in the dialog, or null to close it. */
  submit: (mode: CherryPickMode) => Promise<string | null>;
}

export interface RemoteRequest {
  /** null: add a new remote. Otherwise: change the URL of this one (the name is fixed). */
  existing: { name: string; url: string } | null;
  /** When adding: suggested name, e.g. "origin" when the repo has no remote yet. */
  initialName: string;
  /** Returns an error message to show in the dialog, or null to close it. */
  submit: (name: string, url: string) => Promise<string | null>;
}

export interface SignInRequest {
  /** "github.com". */
  server: string;
  /** Prefilled: the saved account's user name, or "". */
  username: string;
  /** What git said, e.g. "github.com needs a user name and a password or token." */
  reason: string;
  /** Saves the login and retries. Returns an error message to show in the dialog, or null to close it. */
  submit: (username: string, token: string) => Promise<string | null>;
  /** The dialog was closed without signing in. */
  cancel: () => void;
}

export interface AccessRequest {
  repoId: string;
  repoName: string;
}

export interface GroupIdentityRequest {
  groupId: string;
}

export interface IdentityRequest {
  repoId: string;
  repoName: string;
  /** Called with the new identity after Save (e.g. to refresh the Commit dialog). */
  saved?: () => void;
}

export interface CloneRequest {
  /** Group the cloned repo goes into; null = top level. */
  groupId: string | null;
}

export const useDialogStore = defineStore("dialogs", () => {
  const nameRequest = ref<NameRequest | null>(null);
  const cloneRequest = ref<CloneRequest | null>(null);
  const mergeRequest = ref<MergeRequest | null>(null);
  const resetRequest = ref<ResetRequest | null>(null);
  const cherryPickRequest = ref<CherryPickRequest | null>(null);
  const remoteRequest = ref<RemoteRequest | null>(null);
  const signInRequest = ref<SignInRequest | null>(null);
  const identityRequest = ref<IdentityRequest | null>(null);
  const groupIdentityRequest = ref<GroupIdentityRequest | null>(null);
  // Repository → Login (SSH & HTTPS)… (step 15a).
  const accessRequest = ref<AccessRequest | null>(null);
  // MoonGit → Accounts… (the saved logins).
  const accountsOpen = ref(false);

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

  function askReset(request: ResetRequest) {
    resetRequest.value = request;
  }

  function closeReset() {
    resetRequest.value = null;
  }

  function askCherryPick(request: CherryPickRequest) {
    cherryPickRequest.value = request;
  }

  function closeCherryPick() {
    cherryPickRequest.value = null;
  }

  function askRemote(request: RemoteRequest) {
    remoteRequest.value = request;
  }

  function closeRemote() {
    remoteRequest.value = null;
  }

  function askSignIn(request: SignInRequest) {
    signInRequest.value = request;
  }

  function closeSignIn() {
    signInRequest.value = null;
  }

  function askIdentity(request: IdentityRequest) {
    identityRequest.value = request;
  }

  function closeIdentity() {
    identityRequest.value = null;
  }

  function askAccess(request: AccessRequest) {
    accessRequest.value = request;
  }

  function closeAccess() {
    accessRequest.value = null;
  }

  function askGroupIdentity(request: GroupIdentityRequest) {
    groupIdentityRequest.value = request;
  }

  function closeGroupIdentity() {
    groupIdentityRequest.value = null;
  }

  function openAccounts() {
    accountsOpen.value = true;
  }

  function closeAccounts() {
    accountsOpen.value = false;
  }

  return {
    nameRequest,
    cloneRequest,
    mergeRequest,
    resetRequest,
    cherryPickRequest,
    remoteRequest,
    signInRequest,
    accountsOpen,
    identityRequest,
    groupIdentityRequest,
    accessRequest,
    askName,
    closeName,
    openClone,
    closeClone,
    askMerge,
    closeMerge,
    askReset,
    closeReset,
    askCherryPick,
    closeCherryPick,
    askRemote,
    closeRemote,
    askSignIn,
    closeSignIn,
    openAccounts,
    closeAccounts,
    askIdentity,
    closeIdentity,
    askGroupIdentity,
    closeGroupIdentity,
    askAccess,
    closeAccess,
  };
});

// During development, let hot reload replace this store's code too. Without
// it, the store created before an edit keeps its old functions until a full
// reload (⌘R), and newly added ones are missing.
if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useDialogStore, import.meta.hot));
}
