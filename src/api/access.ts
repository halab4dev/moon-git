import { invoke } from "@tauri-apps/api/core";
import type { RemoteAccess } from "../types/access";

// SSH key and HTTPS login of a repository (step 15a).

export function getRemoteAccess(repoId: string): Promise<RemoteAccess> {
  return invoke<RemoteAccess>("get_remote_access", { repoId });
}

/** `sshKey` null: this computer's SSH settings. */
export function setRemoteAccess(repoId: string, sshKey: string | null, systemCredentials: boolean): Promise<RemoteAccess> {
  return invoke<RemoteAccess>("set_remote_access", { repoId, sshKey, systemCredentials });
}
