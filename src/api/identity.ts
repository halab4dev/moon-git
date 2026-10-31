import { invoke } from "@tauri-apps/api/core";
import type { Identity } from "../types/identity";
import type { RepoList } from "../types/repo";

// Who commits are made as, per repository (step 14).

export function getIdentity(repoId: string): Promise<Identity> {
  return invoke<Identity>("get_identity", { repoId });
}

/** `own` null: remove the repository's own identity (the computer-wide one applies). */
export function setIdentity(repoId: string, own: { name: string; email: string } | null): Promise<Identity> {
  return invoke<Identity>("set_identity", { repoId, name: own?.name ?? null, email: own?.email ?? null });
}

/** The computer-wide identity (~/.gitconfig): `inheritedName` / `inheritedEmail`. */
export function getComputerIdentity(): Promise<Identity> {
  return invoke<Identity>("get_computer_identity");
}

export interface GroupIdentityResult {
  list: RepoList;
  /** Repositories in the group (and its subgroups). */
  total: number;
  /** Repositories whose .git/config was changed. */
  changed: number;
  /** "repo name: reason" for each one that couldn't be changed. */
  failed: string[];
}

/**
 * Sets a group's identity (step 14b) and writes it into its repositories:
 * all of them (`all`), or only those without an identity of their own.
 */
export function applyGroupIdentity(
  groupId: string,
  name: string,
  email: string,
  all: boolean,
): Promise<GroupIdentityResult> {
  return invoke<GroupIdentityResult>("apply_group_identity", { groupId, name, email, all });
}
