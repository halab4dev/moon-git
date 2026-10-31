import { invoke } from "@tauri-apps/api/core";
import type { RemoteAccess } from "../types/access";
import type { GroupApplyResult } from "./identity";

// SSH key and HTTPS login of a repository (step 15a).

export function getRemoteAccess(repoId: string): Promise<RemoteAccess> {
  return invoke<RemoteAccess>("get_remote_access", { repoId });
}

/** `sshKey` null: this computer's SSH settings. */
export function setRemoteAccess(repoId: string, sshKey: string | null, systemCredentials: boolean): Promise<RemoteAccess> {
  return invoke<RemoteAccess>("set_remote_access", { repoId, sshKey, systemCredentials });
}

/**
 * Writes login settings into the repositories of a group (step 15b): only
 * into parts a repository hasn't set itself, or into all of them (`all`).
 */
export function applyGroupAccess(
  groupId: string,
  sshKey: string | null,
  systemCredentials: boolean,
  all: boolean,
): Promise<GroupApplyResult> {
  return invoke<GroupApplyResult>("apply_group_access", { groupId, sshKey, systemCredentials, all });
}
