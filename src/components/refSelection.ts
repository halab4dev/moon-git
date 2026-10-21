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
