import { describe, expect, it } from "vitest";
import { currentBranchRefs, readSavedSelection, renameInSavedSelection, saveSelection } from "./refSelection";
import type { RefInfo } from "../types/log";

function info(fullName: string, kind: RefInfo["kind"], current = false, upstream: string | null = null): RefInfo {
  return { fullName, name: fullName.replace(/^refs\/(heads|remotes|tags)\//, ""), kind, current, upstream };
}

describe("currentBranchRefs", () => {
  it("picks the current branch and its upstream", () => {
    const refs = [
      info("refs/heads/feature", "branch", false, "refs/remotes/origin/feature"),
      info("refs/heads/main", "branch", true, "refs/remotes/origin/main"),
      info("refs/remotes/origin/feature", "remote"),
      info("refs/remotes/origin/main", "remote"),
      info("refs/tags/v1", "tag"),
    ];
    expect(currentBranchRefs(refs)).toEqual(["refs/heads/main", "refs/remotes/origin/main"]);
  });

  it("skips an upstream that does not exist (deleted on the server and pruned)", () => {
    const refs = [info("refs/heads/main", "branch", true, "refs/remotes/origin/main")];
    expect(currentBranchRefs(refs)).toEqual(["refs/heads/main"]);
  });

  it("uses HEAD when it is detached", () => {
    const refs = [info("HEAD", "head", true), info("refs/heads/main", "branch")];
    expect(currentBranchRefs(refs)).toEqual(["HEAD"]);
  });

  it("returns nothing for a repository without commits", () => {
    expect(currentBranchRefs([])).toEqual([]);
  });
});

describe("renameInSavedSelection", () => {
  // Vitest runs in Node, which has no localStorage: a small stand-in.
  const store = new Map<string, string>();
  globalThis.localStorage = {
    getItem: (key: string) => store.get(key) ?? null,
    setItem: (key: string, value: string) => void store.set(key, value),
  } as Storage;

  it("replaces the old name and keeps the others", () => {
    saveSelection("r1", ["refs/heads/main", "refs/remotes/origin/main"]);
    renameInSavedSelection("r1", "refs/heads/main", "refs/heads/trunk");
    expect(readSavedSelection("r1")).toEqual(["refs/heads/trunk", "refs/remotes/origin/main"]);
  });

  it("leaves 'all' and 'never chosen' alone", () => {
    saveSelection("r2", null);
    renameInSavedSelection("r2", "refs/heads/main", "refs/heads/trunk");
    expect(readSavedSelection("r2")).toBeNull();

    renameInSavedSelection("r3", "refs/heads/main", "refs/heads/trunk");
    expect(readSavedSelection("r3")).toBeUndefined();
  });
});
