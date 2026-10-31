// Mirror the structs in src-tauri/src/models.rs.

export interface RepoEntry {
  id: string;
  name: string;
  path: string;
  /** null = top level */
  groupId: string | null;
}

export interface RepoGroup {
  id: string;
  name: string;
  /** null = top level */
  parentId: string | null;
  /** Commit identity for the repositories inside (step 14b); missing = none. */
  identity?: GroupIdentity;
  /** Login settings last written for the repositories inside (step 15b); missing = never set. */
  access?: GroupAccess;
}

export interface GroupAccess {
  /** null = this computer's SSH settings. */
  sshKey: string | null;
  systemCredentials: boolean;
}

export interface GroupIdentity {
  name: string;
  email: string;
}

export interface RepoList {
  groups: RepoGroup[];
  repos: RepoEntry[];
}
