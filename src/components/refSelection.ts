import type { RefInfo } from "../types/log";

/**
 * Full names of the checked-out branch (or the detached HEAD) and of the
 * branch it tracks, if that one exists locally. Used for the "Current branch"
 * choice and as the first-time default of the History tab.
 */
export function currentBranchRefs(refs: readonly RefInfo[]): string[] {
  const known = new Set(refs.map((r) => r.fullName));
  const names: string[] = [];
  for (const info of refs) {
    if (!info.current) {
      continue;
    }
    names.push(info.fullName);
    if (info.upstream !== null && known.has(info.upstream)) {
      names.push(info.upstream);
    }
  }
  return names;
}

// ---- the History tab's ref choice, remembered per repository ----

function storageKey(repoId: string): string {
  return `moongit.history.refs.${repoId}`;
}

/**
 * The remembered choice: full ref names, null for "all", or undefined when
 * nothing was chosen yet for this repository (or it can't be read).
 */
export function readSavedSelection(repoId: string): string[] | null | undefined {
  try {
    const stored = localStorage.getItem(storageKey(repoId));
    if (stored === null) {
      return undefined;
    }
    const parsed = JSON.parse(stored);
    if (parsed === null) {
      return null; // "All" was chosen
    }
    if (Array.isArray(parsed) && parsed.every((name) => typeof name === "string")) {
      return parsed;
    }
  } catch {
    // Unreadable: treat as never chosen.
  }
  return undefined;
}

export function saveSelection(repoId: string, selection: string[] | null) {
  try {
    localStorage.setItem(storageKey(repoId), JSON.stringify(selection));
  } catch {
    // Not remembered; fine.
  }
}

/** After renaming a branch, the remembered choice follows the new name. */
export function renameInSavedSelection(repoId: string, oldFullName: string, newFullName: string) {
  const saved = readSavedSelection(repoId);
  if (Array.isArray(saved) && saved.includes(oldFullName)) {
    saveSelection(
      repoId,
      saved.map((name) => (name === oldFullName ? newFullName : name)),
    );
  }
}
