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
}

export interface RepoList {
  groups: RepoGroup[];
  repos: RepoEntry[];
}
