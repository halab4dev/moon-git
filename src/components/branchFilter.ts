import type { BranchList } from "../types/branches";

// Branch → Checkout… (step 31b): which branches match what was typed.

/** One row of the Checkout dialog. */
export interface BranchChoice {
  /** "refs/heads/main" or "refs/remotes/origin/main". */
  fullName: string;
  /** Shown name: "main", or "origin/main" for a remote branch. */
  label: string;
  kind: "local" | "remote";
  /** The branch that is checked out now (listed, but can't be chosen). */
  current: boolean;
  /** Local branches: "↑1 ↓2", "gone" or "". */
  detail: string;
}

const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });

/**
 * Local branches first, then remote ones ("origin/main"), each sorted by
 * name. `query`: every word must appear in the name, in any order and any
 * case, so "log fea" finds "feature/login".
 */
export function filterBranches(list: BranchList, query: string): BranchChoice[] {
  const words = query.toLowerCase().split(/\s+/).filter((word) => word !== "");
  const matches = (label: string) => words.every((word) => label.toLowerCase().includes(word));
  const byLabel = (a: BranchChoice, b: BranchChoice) => collator.compare(a.label, b.label);

  const local: BranchChoice[] = list.local.map((branch) => ({
    fullName: branch.fullName,
    label: branch.name,
    kind: "local",
    current: branch.current,
    detail: branch.upstreamGone
      ? "gone"
      : [branch.ahead > 0 ? `↑${branch.ahead}` : "", branch.behind > 0 ? `↓${branch.behind}` : ""].filter(Boolean).join(" "),
  }));
  const remote: BranchChoice[] = list.remotes.flatMap((group) =>
    group.branches.map((branch) => ({
      fullName: branch.fullName,
      label: `${group.name}/${branch.name}`,
      kind: "remote" as const,
      current: false,
      detail: "",
    })),
  );
  return [
    ...local.filter((choice) => matches(choice.label)).sort(byLabel),
    ...remote.filter((choice) => matches(choice.label)).sort(byLabel),
  ];
}
