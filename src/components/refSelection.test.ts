import { describe, expect, it } from "vitest";
import { currentBranchRefs } from "./refSelection";
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
