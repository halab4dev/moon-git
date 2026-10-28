import { invoke } from "@tauri-apps/api/core";
import type { Account } from "../types/account";

// Saved logins for HTTPS servers (step 13a).

export function listAccounts(): Promise<Account[]> {
  return invoke<Account[]>("list_accounts");
}

/** The account git uses for `host`, or null. */
export function findAccount(host: string): Promise<Account | null> {
  return invoke<Account | null>("find_account", { host });
}

/**
 * Saves a login before it is tried (git can only use what is saved). Retry
 * the git command, then call `finishSignIn`. Resolves with the account id.
 */
export function beginSignIn(host: string, username: string, token: string): Promise<string> {
  return invoke<string>("begin_sign_in", { host, username, token });
}

/** `accepted` false: the server refused it, the previous state comes back. */
export function finishSignIn(id: string, accepted: boolean): Promise<void> {
  return invoke<void>("finish_sign_in", { id, accepted });
}
