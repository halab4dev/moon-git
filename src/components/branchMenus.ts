import { Menu } from "@tauri-apps/api/menu";
import { checkoutBranch, createBranch } from "../api/branch";
import { toAppError } from "../api/errors";
import { listRefs } from "../api/log";
import { useDialogStore } from "../stores/dialogs";
import type { AppError } from "../types/error";
import type { CommitSummary } from "../types/log";
import type { RepoStatus } from "../types/status";

// Native menus and the "New Branch" dialog, used from the branch name in the
// repo header and from right-clicking a commit in the History tab.
// Like sidebarMenus.ts: menu actions can't be awaited, so each starts a task.

/** Where results go: the new status after a change, or an error to show. */
export interface BranchCallbacks {
  onStatus: (status: RepoStatus) => void;
  onError: (error: AppError) => void;
}

/** Runs a branch action and reports its outcome through `callbacks`. */
async function perform(action: () => Promise<RepoStatus>, callbacks: BranchCallbacks) {
  try {
    callbacks.onStatus(await action());
  } catch (err) {
    callbacks.onError(toAppError(err));
  }
}

/**
 * The menu of the branch name in the repo header: every local branch (a
 * check mark on the current one), the remote branches in a submenu, and
 * "New Branch…".
 */
export async function showBranchMenu(repoId: string, callbacks: BranchCallbacks) {
  let refs;
  try {
    refs = await listRefs(repoId);
  } catch (err) {
    callbacks.onError(toAppError(err));
    return;
  }
  const local = refs.filter((r) => r.kind === "branch");
  const remote = refs.filter((r) => r.kind === "remote");

  const menu = await Menu.new({
    items: [
      // "checked" makes these check-mark items; clicking one checks it out.
      ...local.map((info) => ({
        text: info.name,
        checked: info.current,
        action: () => {
          if (!info.current) {
            void perform(() => checkoutBranch(repoId, info.fullName), callbacks);
          }
        },
      })),
      { item: "Separator" as const },
      // Only when there are any: an empty submenu is not allowed on every platform.
      ...(remote.length > 0
        ? [
            {
              text: "Remote Branches",
              items: remote.map((info) => ({
                text: info.name,
                action: () => void perform(() => checkoutBranch(repoId, info.fullName), callbacks),
              })),
            },
            { item: "Separator" as const },
          ]
        : []),
      { text: "New Branch…", action: () => askNewBranch(repoId, null, "the current commit", callbacks) },
    ],
  });
  await menu.popup();
}

/** Right-click on a commit in the History tab. */
export async function showCommitMenu(repoId: string, commit: CommitSummary, callbacks: BranchCallbacks) {
  const branches = commit.refs.filter((r) => (r.kind === "branch" && !r.current) || r.kind === "remote");
  const menu = await Menu.new({
    items: [
      {
        text: "Create Branch Here…",
        action: () => askNewBranch(repoId, commit.id, commit.id.slice(0, 8), callbacks),
      },
      ...(branches.length > 0 ? [{ item: "Separator" as const }] : []),
      ...branches.map((r) => ({
        text: `Check Out ${r.name}`,
        action: () => {
          // Labels carry short names; the full name is rebuilt from the kind.
          const fullName = r.kind === "branch" ? `refs/heads/${r.name}` : `refs/remotes/${r.name}`;
          void perform(() => checkoutBranch(repoId, fullName), callbacks);
        },
      })),
    ],
  });
  await menu.popup();
}

/**
 * "New Branch" dialog. `start`: commit id (or full ref name) to start from;
 * null = the current commit. `startLabel` says where in the title.
 */
export function askNewBranch(repoId: string, start: string | null, startLabel: string, callbacks: BranchCallbacks) {
  useDialogStore().askName({
    title: `New Branch at ${startLabel}`,
    confirmLabel: "Create",
    initial: "",
    option: { label: "Check out the new branch", initial: true },
    submit: async (name, checkOut) => {
      try {
        callbacks.onStatus(await createBranch(repoId, name, start, checkOut));
        return null;
      } catch (err) {
        // Shown inside the dialog, which stays open to fix the name.
        return toAppError(err).message;
      }
    },
  });
}
