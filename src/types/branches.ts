// Mirror BranchList and friends in src-tauri/src/models.rs.

export interface LocalBranch {
  name: string;
  /** "refs/heads/main". */
  fullName: string;
  current: boolean;
  /** e.g. "origin/main"; null when it tracks nothing. */
  upstream: string | null;
  ahead: number;
  behind: number;
  /** The upstream was deleted on the server. */
  upstreamGone: boolean;
}

export interface RemoteBranch {
  /** Without the remote: "main", "feature/x". */
  name: string;
  /** "refs/remotes/origin/main". */
  fullName: string;
}

export interface RemoteGroup {
  /** "origin", "upstream". */
  name: string;
  branches: RemoteBranch[];
}

export interface BranchList {
  local: LocalBranch[];
  remotes: RemoteGroup[];
}
