import type { ChangeKind, FileChange, RepoStatus } from "../types/status";

// Turns the four lists from `git status` into one row per file for the
// file table, and sorts those rows. Plain functions, no Vue, easy to test.

export interface FileRow {
  path: string;
  /** File name, e.g. "App.vue". */
  name: string;
  /** Folder relative to the repo root, e.g. "src/views"; "" for the root. */
  location: string;
  /** Previous path of a renamed or copied file. */
  origPath: string | null;
  /** Kind used for the colored letter. */
  kind: ChangeKind;
  /** Readable state, e.g. "Modified (partly staged)". */
  state: string;
  hasStaged: boolean;
  hasUnstaged: boolean;
}

export type SortKey = "name" | "state" | "location";

export const KIND_LETTER: Record<ChangeKind, string> = {
  added: "A",
  modified: "M",
  deleted: "D",
  renamed: "R",
  copied: "C",
  typeChanged: "T",
  unmerged: "U",
  untracked: "?",
};

const KIND_LABEL: Record<ChangeKind, string> = {
  added: "Added",
  modified: "Modified",
  deleted: "Deleted",
  renamed: "Renamed",
  copied: "Copied",
  typeChanged: "Type changed",
  unmerged: "Conflict",
  untracked: "Untracked",
};

export function buildRows(status: RepoStatus): FileRow[] {
  const rows: FileRow[] = [];

  for (const file of status.conflicted) {
    rows.push(makeRow(file, KIND_LABEL.unmerged, false, true));
  }

  // A file changed in both the index and the working tree comes twice from
  // git; merge the two into one row.
  const unstagedByPath = new Map(status.unstaged.map((f) => [f.path, f]));
  for (const staged of status.staged) {
    const unstaged = unstagedByPath.get(staged.path);
    unstagedByPath.delete(staged.path);
    rows.push(makeRow(staged, describe(staged, unstaged), true, unstaged !== undefined));
  }
  for (const unstaged of unstagedByPath.values()) {
    rows.push(makeRow(unstaged, KIND_LABEL[unstaged.kind], false, true));
  }

  for (const file of status.untracked) {
    rows.push(makeRow(file, KIND_LABEL.untracked, false, true));
  }
  return rows;
}

function describe(staged: FileChange, unstaged: FileChange | undefined): string {
  const stagedLabel = KIND_LABEL[staged.kind];
  if (unstaged === undefined) {
    return `${stagedLabel} (staged)`;
  }
  if (unstaged.kind === staged.kind) {
    return `${stagedLabel} (partly staged)`;
  }
  return `${stagedLabel} (staged), ${KIND_LABEL[unstaged.kind].toLowerCase()}`;
}

function makeRow(file: FileChange, state: string, hasStaged: boolean, hasUnstaged: boolean): FileRow {
  const slash = file.path.lastIndexOf("/");
  return {
    path: file.path,
    name: slash === -1 ? file.path : file.path.slice(slash + 1),
    location: slash === -1 ? "" : file.path.slice(0, slash),
    origPath: file.origPath,
    kind: file.kind,
    state,
    hasStaged,
    hasUnstaged,
  };
}

// numeric: "file10" after "file9"; sensitivity "base": ignore upper/lower case.
const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });

// When two rows are equal on the chosen column, the remaining columns decide,
// so the order is always stable and predictable.
const TIE_BREAKERS: Record<SortKey, SortKey[]> = {
  name: ["name", "location"],
  state: ["state", "location", "name"],
  location: ["location", "name"],
};

/** Returns a sorted copy; the input array is not changed. */
export function sortRows(rows: FileRow[], key: SortKey, ascending: boolean): FileRow[] {
  const keys = TIE_BREAKERS[key];
  const direction = ascending ? 1 : -1;
  return [...rows].sort((a, b) => {
    for (const k of keys) {
      const result = collator.compare(a[k], b[k]);
      if (result !== 0) {
        return result * direction;
      }
    }
    return 0;
  });
}
