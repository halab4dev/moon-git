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

/** What writing a group's identity (14b) or login (15b) into its repositories did. */
export interface GroupApplyResult {
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
): Promise<GroupApplyResult> {
  return invoke<GroupApplyResult>("apply_group_identity", { groupId, name, email, all });
}

/** The same for repositories picked in the sidebar (step 35c). */
export function applyIdentityToRepos(repoIds: string[], name: string, email: string, all: boolean): Promise<GroupApplyResult> {
  return invoke<GroupApplyResult>("apply_identity_to_repos", { repoIds, name, email, all });
}
