import { Menu } from "@tauri-apps/api/menu";
import { ask, open } from "@tauri-apps/plugin-dialog";
import { toAppError } from "../api/errors";
import { useDialogStore } from "../stores/dialogs";
import { useRepoStore } from "../stores/repos";
import type { RepoEntry, RepoGroup } from "../types/repo";

// Native menus for the sidebar ("+" button and right-click).
// Menu actions can't be awaited by the menu, so each one starts an async task.

const separator = { item: "Separator" } as const;

/** "+" button, or right-click on empty space. `groupId` is where new items go. */
export async function showAddMenu(groupId: string | null) {
  const menu = await Menu.new({
    items: [
      { text: "Add Local Repository…", action: () => void addLocalRepo(groupId) },
      { text: "Clone Repository…", action: () => useDialogStore().openClone(groupId) },
      separator,
      { text: groupId ? "New Subgroup…" : "New Group…", action: () => newGroup(groupId) },
    ],
  });
  await menu.popup();
}

export async function showGroupMenu(group: RepoGroup) {
  const menu = await Menu.new({
    items: [
      { text: "Add Local Repository Here…", action: () => void addLocalRepo(group.id) },
      { text: "Clone Repository Here…", action: () => useDialogStore().openClone(group.id) },
      { text: "New Subgroup…", action: () => newGroup(group.id) },
      separator,
      { text: "Rename…", action: () => renameGroup(group) },
      { text: "Commit Identity…", action: () => useDialogStore().askGroupIdentity({ groupId: group.id }) },
      { text: "Login (SSH & HTTPS)…", action: () => useDialogStore().askGroupAccess({ groupId: group.id }) },
      { text: "Delete Group…", action: () => void deleteGroup(group) },
    ],
  });
  await menu.popup();
}

export async function showRepoMenu(repo: RepoEntry) {
  const store = useRepoStore();
  // Right-click on one of several picked repositories: act on all of them (step 35b).
  if (store.isPicked(repo.id) && store.pickedRepos.length > 1) {
    await showPickedReposMenu([...store.pickedRepos]);
    return;
  }
  const moveTo = (groupId: string | null) => () => void store.attempt(() => store.moveRepo(repo.id, groupId));

  const menu = await Menu.new({
    items: [
      {
        text: "Move To",
        items: [
          { text: "Top Level", enabled: repo.groupId !== null, action: moveTo(null) },
          ...(store.groupOptions.length > 0 ? [separator] : []),
          ...store.groupOptions.map((option) => ({
            text: option.label,
            enabled: option.id !== repo.groupId,
            action: moveTo(option.id),
          })),
        ],
      },
      separator,
      {
        text: "Commit Identity…",
        action: () => useDialogStore().askIdentity({ repoId: repo.id, repoName: repo.name }),
      },
      {
        text: "Login (SSH & HTTPS)…",
        action: () => useDialogStore().askAccess({ repoId: repo.id, repoName: repo.name }),
      },
      separator,
      // Only forgets the repo; the folder stays on disk, so no confirmation.
      { text: "Remove from List", action: () => void store.attempt(() => store.removeRepo(repo.id)) },
    ],
  });
  await menu.popup();
}

/** The menu for several picked repositories: the actions that make sense for all of them. */
async function showPickedReposMenu(repos: RepoEntry[]) {
  const store = useRepoStore();
  const moveAll = (groupId: string | null) => () =>
    void store.attempt(async () => {
      for (const repo of repos.filter((r) => r.groupId !== groupId)) {
        await store.moveRepo(repo.id, groupId);
      }
    });

  const menu = await Menu.new({
    items: [
      { text: `${repos.length} Repositories`, enabled: false },
      separator,
      {
        text: "Move To",
        items: [
          { text: "Top Level", enabled: repos.some((r) => r.groupId !== null), action: moveAll(null) },
          ...(store.groupOptions.length > 0 ? [separator] : []),
          ...store.groupOptions.map((option) => ({
            text: option.label,
            enabled: repos.some((r) => r.groupId !== option.id),
            action: moveAll(option.id),
          })),
        ],
      },
      separator,
      {
        text: "Commit Identity…",
        action: () => useDialogStore().askGroupIdentity({ repoIds: repos.map((r) => r.id) }),
      },
      {
        text: "Login (SSH & HTTPS)…",
        action: () => useDialogStore().askGroupAccess({ repoIds: repos.map((r) => r.id) }),
      },
      separator,
      { text: `Remove ${repos.length} from List…`, action: () => void removePicked(repos) },
    ],
  });
  await menu.popup();
}

// Several at once: asked first (one is not, as it is easy to add back).
async function removePicked(repos: RepoEntry[]) {
  const store = useRepoStore();
  const names = repos.slice(0, 10).map((r) => `• ${r.name}`);
  if (repos.length > 10) {
    names.push(`… and ${repos.length - 10} more`);
  }
  const confirmed = await ask(
    `Remove ${repos.length} repositories from the list?\n\n${names.join("\n")}\n\nOnly the list changes: the folders stay on disk.`,
    { title: "Remove from List", kind: "warning", okLabel: "Remove", cancelLabel: "Cancel" },
  );
  if (!confirmed) {
    return;
  }
  await store.attempt(async () => {
    for (const repo of repos) {
      await store.removeRepo(repo.id);
    }
  });
  store.clearPicked();
}

export async function addLocalRepo(groupId: string | null) {
  const store = useRepoStore();
  const picked = await open({ directory: true, multiple: false, title: "Add Local Repository" });
  // null means the user cancelled the dialog.
  if (typeof picked === "string") {
    await store.attempt(() => store.addRepo(picked, groupId));
  }
}

export function newGroup(parentId: string | null) {
  const store = useRepoStore();
  useDialogStore().askName({
    title: parentId ? "New Subgroup" : "New Group",
    confirmLabel: "Create",
    initial: "",
    submit: (name) => messageOnError(() => store.createGroup(name, parentId)),
  });
}

function renameGroup(group: RepoGroup) {
  const store = useRepoStore();
  useDialogStore().askName({
    title: "Rename Group",
    confirmLabel: "Rename",
    initial: group.name,
    submit: (name) => messageOnError(() => store.renameGroup(group.id, name)),
  });
}

async function deleteGroup(group: RepoGroup) {
  const store = useRepoStore();
  const confirmed = await ask(
    `Delete the group "${group.name}"?\n\nRepositories and subgroups inside it move up one level. Nothing is deleted from disk.`,
    { title: "Delete Group", kind: "warning", okLabel: "Delete", cancelLabel: "Cancel" },
  );
  if (confirmed) {
    await store.attempt(() => store.deleteGroup(group.id));
  }
}

// For the name dialog: null closes it, a message keeps it open and shows the error.
async function messageOnError(action: () => Promise<void>): Promise<string | null> {
  try {
    await action();
    return null;
  } catch (err) {
    return toAppError(err).message;
  }
}
