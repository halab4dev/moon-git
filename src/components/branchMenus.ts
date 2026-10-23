import { Menu } from "@tauri-apps/api/menu";
import { ask, message } from "@tauri-apps/plugin-dialog";
import {
  checkoutBranch,
  createBranch,
  deleteBranch,
  deleteRemoteBranch,
  renameBranch,
  setBranchUpstream,
} from "../api/branch";
import { toAppError } from "../api/errors";
import { listRefs } from "../api/log";
import { getMergePreview, mergeIntoCurrent } from "../api/merge";
import { getRebasePreview, rebaseCurrent } from "../api/rebase";
import { getStatus } from "../api/status";
import { useDialogStore } from "../stores/dialogs";
import { renameInSavedSelection } from "./refSelection";
import type { AppError } from "../types/error";
import type { CommitRef, CommitSummary, RefInfo } from "../types/log";
import type { RepoStatus } from "../types/status";

// Native menus and dialogs for branches, used from the branch name in the
// repo header and from right-clicking a commit in the History tab.
// Like sidebarMenus.ts: menu actions can't be awaited, so each starts a task.
//
// Everything that deletes asks first (CLAUDE.md: no destructive git command
// without a confirmation in the UI).

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

const SEPARATOR = { item: "Separator" as const };

/**
 * The menu of the branch name in the repo header: every local branch (a
 * check mark on the current one) to check out, the remote branches in a
 * submenu, then New / Rename / Upstream / Delete.
 */
export async function showBranchMenu(repoId: string, callbacks: BranchCallbacks) {
  let refs: RefInfo[];
  try {
    refs = await listRefs(repoId);
  } catch (err) {
    callbacks.onError(toAppError(err));
    return;
  }
  const local = refs.filter((r) => r.kind === "branch");
  const remote = refs.filter((r) => r.kind === "remote");
  const current = local.find((r) => r.current) ?? null;
  const others = local.filter((r) => !r.current);

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
      SEPARATOR,
      // Submenus only when they have entries: an empty submenu is not allowed everywhere.
      ...(remote.length > 0
        ? [
            {
              text: "Remote Branches",
              items: remote.map((info) => ({
                text: info.name,
                action: () => void perform(() => checkoutBranch(repoId, info.fullName), callbacks),
              })),
            },
            SEPARATOR,
          ]
        : []),
      { text: "New Branch…", action: () => askNewBranch(repoId, null, "the current commit", callbacks) },
      ...(current !== null
        ? [
            { text: `Rename "${current.name}"…`, action: () => askRename(repoId, current, callbacks) },
            ...upstreamMenu(repoId, current, remote, callbacks),
            ...mergeMenu(repoId, current, [...others, ...remote], callbacks),
            ...rebaseMenu(repoId, current, [...others, ...remote], callbacks),
          ]
        : []),
      ...(others.length > 0 || remote.length > 0 ? [SEPARATOR] : []),
      ...(others.length > 0
        ? [
            {
              text: "Delete Branch",
              items: others.map((info) => ({
                text: info.name,
                action: () => void confirmDeleteLocal(repoId, info, callbacks),
              })),
            },
          ]
        : []),
      ...(remote.length > 0
        ? [
            {
              text: "Delete Remote Branch",
              items: remote.map((info) => ({
                text: info.name,
                action: () => void confirmDeleteRemote(repoId, info, callbacks),
              })),
            },
          ]
        : []),
    ],
  });
  await menu.popup();
}

// "Upstream of main ▸": every remote branch to track (check mark on the
// current one), and "Stop Tracking".
function upstreamMenu(repoId: string, branch: RefInfo, remote: RefInfo[], callbacks: BranchCallbacks) {
  if (remote.length === 0 && branch.upstream === null) {
    return [];
  }
  const choose = (upstream: string | null) =>
    void perform(() => setBranchUpstream(repoId, branch.fullName, upstream), callbacks);
  return [
    {
      text: `Upstream of "${branch.name}"`,
      items: [
        ...remote.map((info) => ({
          text: info.name,
          checked: info.fullName === branch.upstream,
          action: () => choose(info.fullName),
        })),
        ...(remote.length > 0 ? [SEPARATOR] : []),
        { text: "Stop Tracking", enabled: branch.upstream !== null, action: () => choose(null) },
      ],
    },
  ];
}

// "Merge into main ▸": the other local branches and the remote branches.
function mergeMenu(repoId: string, current: RefInfo, sources: RefInfo[], callbacks: BranchCallbacks) {
  if (sources.length === 0) {
    return [];
  }
  return [
    {
      text: `Merge into "${current.name}"`,
      items: sources.map((info) => ({
        text: `${info.name}…`,
        action: () => void askMerge(repoId, info.fullName, callbacks),
      })),
    },
  ];
}

// "Rebase main onto ▸": the same targets as merging.
function rebaseMenu(repoId: string, current: RefInfo, targets: RefInfo[], callbacks: BranchCallbacks) {
  if (targets.length === 0) {
    return [];
  }
  return [
    {
      text: `Rebase "${current.name}" onto`,
      items: targets.map((info) => ({
        text: `${info.name}…`,
        action: () => void confirmRebase(repoId, info.fullName, callbacks),
      })),
    },
  ];
}

/** Right-click on a commit in the History tab. */
export async function showCommitMenu(repoId: string, commit: CommitSummary, callbacks: BranchCallbacks) {
  const local = commit.refs.filter((r) => r.kind === "branch");
  const remote = commit.refs.filter((r) => r.kind === "remote");
  const checkoutable = [...local.filter((r) => !r.current), ...remote];

  const menu = await Menu.new({
    items: [
      {
        text: "Create Branch Here…",
        action: () => askNewBranch(repoId, commit.id, commit.id.slice(0, 8), callbacks),
      },
      ...(checkoutable.length > 0 ? [SEPARATOR] : []),
      ...checkoutable.map((r) => ({
        text: `Check Out ${r.name}`,
        action: () => void perform(() => checkoutBranch(repoId, fullNameOf(r)), callbacks),
      })),
      ...(local.length + remote.length > 0 ? [SEPARATOR] : []),
      ...local.map((r) => ({
        text: `Rename ${r.name}…`,
        action: () => askRename(repoId, toRefInfo(r), callbacks),
      })),
      ...local
        .filter((r) => !r.current)
        .map((r) => ({
          text: `Delete ${r.name}…`,
          action: () => void confirmDeleteLocal(repoId, toRefInfo(r), callbacks),
        })),
      ...remote.map((r) => ({
        text: `Delete ${r.name} on the Server…`,
        action: () => void confirmDeleteRemote(repoId, toRefInfo(r), callbacks),
      })),
      SEPARATOR,
      ...mergeItems(repoId, commit, callbacks),
      ...rebaseItems(repoId, commit, callbacks),
    ],
  });
  await menu.popup();
}

// "Merge feature into Current Branch…" for each branch or tag on the commit
// (not the current branch itself), or the bare commit when it has none.
function mergeItems(repoId: string, commit: CommitSummary, callbacks: BranchCallbacks) {
  const named = commit.refs.filter((r) => r.kind !== "head" && !r.current);
  if (named.length === 0) {
    return [
      {
        text: `Merge Commit ${commit.id.slice(0, 8)} into Current Branch…`,
        action: () => void askMerge(repoId, commit.id, callbacks),
      },
    ];
  }
  return named.map((r) => ({
    text: `Merge ${r.name} into Current Branch…`,
    action: () => void askMerge(repoId, fullNameOf(r), callbacks),
  }));
}

// "Rebase Current Branch onto feature…", like mergeItems.
function rebaseItems(repoId: string, commit: CommitSummary, callbacks: BranchCallbacks) {
  const named = commit.refs.filter((r) => r.kind !== "head" && !r.current);
  if (named.length === 0) {
    return [
      {
        text: `Rebase Current Branch onto Commit ${commit.id.slice(0, 8)}…`,
        action: () => void confirmRebase(repoId, commit.id, callbacks),
      },
    ];
  }
  return named.map((r) => ({
    text: `Rebase Current Branch onto ${r.name}…`,
    action: () => void confirmRebase(repoId, fullNameOf(r), callbacks),
  }));
}

// History labels carry short names; the full name follows from the kind.
function fullNameOf(r: CommitRef): string {
  switch (r.kind) {
    case "branch":
      return `refs/heads/${r.name}`;
    case "tag":
      return `refs/tags/${r.name}`;
    default:
      return `refs/remotes/${r.name}`;
  }
}

function toRefInfo(r: CommitRef): RefInfo {
  return { fullName: fullNameOf(r), name: r.name, kind: r.kind, current: r.current, upstream: null };
}

// ---- dialogs ----

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

function askRename(repoId: string, branch: RefInfo, callbacks: BranchCallbacks) {
  useDialogStore().askName({
    title: `Rename Branch "${branch.name}"`,
    confirmLabel: "Rename",
    initial: branch.name,
    submit: async (name) => {
      try {
        const status = await renameBranch(repoId, branch.fullName, name);
        // Before onStatus: the History tab re-reads its saved choice when the status changes.
        renameInSavedSelection(repoId, branch.fullName, `refs/heads/${name.trim()}`);
        callbacks.onStatus(status);
        return null;
      } catch (err) {
        return toAppError(err).message;
      }
    },
  });
}

// Two steps: a normal delete first. If git says the branch has commits found
// nowhere else, ask again, more clearly, before forcing it.
async function confirmDeleteLocal(repoId: string, branch: RefInfo, callbacks: BranchCallbacks) {
  const confirmed = await ask(`Delete the branch "${branch.name}"?\n\nOnly the branch name is removed on this computer.`, {
    title: "Delete Branch",
    kind: "warning",
    okLabel: "Delete",
    cancelLabel: "Cancel",
  });
  if (!confirmed) {
    return;
  }
  try {
    callbacks.onStatus(await deleteBranch(repoId, branch.fullName, false));
    return;
  } catch (err) {
    const error = toAppError(err);
    if (error.kind !== "notMerged") {
      callbacks.onError(error);
      return;
    }
    const force = await ask(`${error.message}\n\nDelete "${branch.name}" anyway?`, {
      title: "Branch Not Merged",
      kind: "warning",
      okLabel: "Delete Anyway",
      cancelLabel: "Keep Branch",
    });
    if (force) {
      await perform(() => deleteBranch(repoId, branch.fullName, true), callbacks);
    }
  }
}

// Shows what the merge would do first (in the dialog), then merges.
async function askMerge(repoId: string, source: string, callbacks: BranchCallbacks) {
  let preview;
  try {
    preview = await getMergePreview(repoId, source);
  } catch (err) {
    callbacks.onError(toAppError(err));
    return;
  }
  if (preview.upToDate) {
    await message(`"${preview.branch}" already contains everything in "${preview.source}".`, {
      title: "Nothing to Merge",
      kind: "info",
    });
    return;
  }
  useDialogStore().askMerge({
    preview,
    submit: async (mode) => {
      try {
        callbacks.onStatus(await mergeIntoCurrent(repoId, source, mode));
        return null;
      } catch (err) {
        const error = toAppError(err);
        if (error.kind !== "conflict") {
          // Nothing changed: show it in the dialog, which stays open.
          return error.message;
        }
        // Conflicts: the merge did change files. Close the dialog, show the
        // new status (conflicted files), then the explanation.
        await perform(() => getStatus(repoId), callbacks);
        callbacks.onError(error);
        return null;
      }
    },
  });
}

// Rebasing rewrites commits, so it always asks first, and says plainly when
// some of those commits are already on the server.
async function confirmRebase(repoId: string, onto: string, callbacks: BranchCallbacks) {
  let preview;
  try {
    preview = await getRebasePreview(repoId, onto);
  } catch (err) {
    callbacks.onError(toAppError(err));
    return;
  }
  const { branch, commits, pushed } = preview;
  if (preview.upToDate) {
    await message(`"${branch}" already starts from "${preview.onto}".`, { title: "Nothing to Rebase", kind: "info" });
    return;
  }
  let text =
    commits === 0
      ? `"${branch}" has no commits of its own: it simply moves forward to "${preview.onto}".`
      : `The ${commits} commit(s) of "${branch}" that are not in "${preview.onto}" will be replayed on top of it, as new commits.`;
  if (pushed > 0) {
    text += `\n\n${pushed} of them are already pushed. Afterwards, pushing "${branch}" needs --force (MoonGit never forces), and anyone who pulled them has to fix their copy.`;
  }
  const confirmed = await ask(text, {
    title: `Rebase "${branch}" onto "${preview.onto}"`,
    kind: pushed > 0 ? "warning" : "info",
    okLabel: "Rebase",
    cancelLabel: "Cancel",
  });
  if (!confirmed) {
    return;
  }
  try {
    callbacks.onStatus(await rebaseCurrent(repoId, onto));
  } catch (err) {
    // On conflicts the rebase changed files and waits halfway: show that
    // status first, then the explanation.
    const error = toAppError(err);
    if (error.kind === "conflict") {
      await perform(() => getStatus(repoId), callbacks);
    }
    callbacks.onError(error);
  }
}

async function confirmDeleteRemote(repoId: string, branch: RefInfo, callbacks: BranchCallbacks) {
  const confirmed = await ask(
    `Delete "${branch.name}" on the server?\n\nThe branch disappears for everyone who uses this repository. Your local branches are not changed.`,
    { title: "Delete Remote Branch", kind: "warning", okLabel: "Delete on Server", cancelLabel: "Cancel" },
  );
  if (confirmed) {
    // No progress shown: deleting a branch transfers almost nothing.
    await perform(() => deleteRemoteBranch(crypto.randomUUID(), repoId, branch.fullName), callbacks);
  }
}
