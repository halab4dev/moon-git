import type { ChangeKind, FileChange, RepoStatus } from "../types/status";

// Turns the four lists from `git status` into one row per file for the
// file table, and sorts those rows. Plain functions, no Vue, easy to test.

/**
 * One state per file, named like SmartGit's file states
 * (https://docs.syntevo.com/SmartGit/Latest/Repositories-Directories-and-Files).
 */
export type FileState =
  | "conflicted"
  | "staged"
  | "stagedModified"
  | "added"
  | "addedModified"
  | "renamed"
  | "renamedModified"
  | "copied"
  | "removed"
  | "modified"
  | "typeChanged"
  | "missing"
  | "intentToAdd"
  | "untracked";

/** Color group of the letter in front of the file name. */
type Tone = "conflict" | "staged" | "renamed" | "removed" | "modified" | "untracked";

interface StateInfo {
  label: string;
  letter: string;
  tone: Tone;
}

// The order of this table is the order used when sorting by State:
// things to deal with first (conflicts, then what is staged for the next
// commit), working-tree changes after, untracked files last.
const STATE_TABLE: [FileState, StateInfo][] = [
  ["conflicted", { label: "Conflicted", letter: "U", tone: "conflict" }],
  ["staged", { label: "Staged", letter: "M", tone: "staged" }],
  ["stagedModified", { label: "Staged Modified", letter: "M", tone: "staged" }],
  ["added", { label: "Added", letter: "A", tone: "staged" }],
  ["addedModified", { label: "Added Modified", letter: "A", tone: "staged" }],
  ["renamed", { label: "Renamed", letter: "R", tone: "renamed" }],
  ["renamedModified", { label: "Renamed (modified)", letter: "R", tone: "renamed" }],
  ["copied", { label: "Copied", letter: "C", tone: "renamed" }],
  ["removed", { label: "Removed", letter: "D", tone: "removed" }],
  ["modified", { label: "Modified", letter: "M", tone: "modified" }],
  ["typeChanged", { label: "Type Changed", letter: "T", tone: "modified" }],
  ["missing", { label: "Missing", letter: "!", tone: "removed" }],
  ["intentToAdd", { label: "Intent-to-Add", letter: "A", tone: "modified" }],
  ["untracked", { label: "Untracked", letter: "?", tone: "untracked" }],
];

export const STATE_INFO = new Map(STATE_TABLE);
const STATE_RANK = new Map(STATE_TABLE.map(([state], index) => [state, index]));

export interface FileRow {
  path: string;
  /** File name, e.g. "App.vue". */
  name: string;
  /** Folder relative to the repo root, e.g. "src/views"; "" for the root. */
  location: string;
  /** Previous path of a renamed or copied file. */
  origPath: string | null;
  state: FileState;
  hasStaged: boolean;
  hasUnstaged: boolean;
}

export type SortKey = "name" | "state" | "location";

export function stateLabel(state: FileState): string {
  return STATE_INFO.get(state)?.label ?? state;
}

export function stateLetter(state: FileState): string {
  return STATE_INFO.get(state)?.letter ?? "";
}

export function stateTone(state: FileState): Tone {
  return STATE_INFO.get(state)?.tone ?? "modified";
}

export function buildRows(status: RepoStatus): FileRow[] {
  const rows: FileRow[] = [];

  for (const file of status.conflicted) {
    rows.push(makeRow(file, "conflicted", false, true));
  }

  // A file changed in both the index and the working tree comes twice from
  // git; merge the two into one row.
  const unstagedByPath = new Map(status.unstaged.map((f) => [f.path, f]));
  for (const staged of status.staged) {
    const unstaged = unstagedByPath.get(staged.path);
    unstagedByPath.delete(staged.path);
    rows.push(makeRow(staged, stateOf(staged.kind, unstaged?.kind ?? null), true, unstaged !== undefined));
  }
  for (const unstaged of unstagedByPath.values()) {
    rows.push(makeRow(unstaged, stateOf(null, unstaged.kind), false, true));
  }

  for (const file of status.untracked) {
    rows.push(makeRow(file, "untracked", false, true));
  }
  return rows;
}

/**
 * `staged` = change in the index (X in `git status`), `unstaged` = change in
 * the working tree (Y); null = no change on that side.
 */
export function stateOf(staged: ChangeKind | null, unstaged: ChangeKind | null): FileState {
  // Whatever the index says, a file that is gone from disk is "Missing".
  if (unstaged === "deleted") {
    return "missing";
  }
  const alsoModified = unstaged !== null;
  switch (staged) {
    case "modified":
      return alsoModified ? "stagedModified" : "staged";
    case "added":
      return alsoModified ? "addedModified" : "added";
    case "renamed":
      return alsoModified ? "renamedModified" : "renamed";
    case "copied":
      return "copied";
    case "deleted":
      return "removed";
    case "typeChanged":
      return "typeChanged";
  }
  // Nothing staged: only a working-tree change.
  if (unstaged === "added") {
    return "intentToAdd"; // `git add -N`
  }
  if (unstaged === "typeChanged") {
    return "typeChanged";
  }
  return "modified";
}

function makeRow(file: FileChange, state: FileState, hasStaged: boolean, hasUnstaged: boolean): FileRow {
  const slash = file.path.lastIndexOf("/");
  return {
    path: file.path,
    name: slash === -1 ? file.path : file.path.slice(slash + 1),
    location: slash === -1 ? "" : file.path.slice(0, slash),
    origPath: file.origPath,
    state,
    hasStaged,
    hasUnstaged,
  };
}

// numeric: "file10" after "file9"; sensitivity "base": ignore upper/lower case.
const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });

function compareBy(key: SortKey, a: FileRow, b: FileRow): number {
  if (key === "state") {
    // By importance (table order above), not alphabetically.
    return (STATE_RANK.get(a.state) ?? 0) - (STATE_RANK.get(b.state) ?? 0);
  }
  return collator.compare(a[key], b[key]);
}

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
      const result = compareBy(k, a, b);
      if (result !== 0) {
        return result * direction;
      }
    }
    return 0;
  });
}
