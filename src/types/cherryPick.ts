// Mirror CherryPickMode and CherryPickAction in src-tauri/src/models.rs.

/** commit: a new commit with the same changes; workingTree: only the changes, staged (--no-commit). */
export type CherryPickMode = "commit" | "workingTree";

export type CherryPickAction = "continue" | "abort";
