import { describe, expect, it } from "vitest";
import { filterBranches } from "./branchFilter";
import type { BranchList } from "../types/branches";

const local = (name: string, extra: Partial<BranchList["local"][number]> = {}) => ({
  name,
  fullName: `refs/heads/${name}`,
  current: false,
  upstream: null,
  ahead: 0,
  behind: 0,
  upstreamGone: false,
  ...extra,
});

const list: BranchList = {
  local: [local("main", { current: true, ahead: 1, behind: 2 }), local("feature/login"), local("Feature-10"), local("feature-2", { upstreamGone: true })],
  remotes: [
    { name: "origin", url: null, branches: [{ name: "main", fullName: "refs/remotes/origin/main" }, { name: "feature/login", fullName: "refs/remotes/origin/feature/login" }] },
    { name: "upstream", url: null, branches: [{ name: "release", fullName: "refs/remotes/upstream/release" }] },
  ],
};

const labels = (query: string) => filterBranches(list, query).map((choice) => choice.label);

describe("filterBranches", () => {
  it("lists local branches first, then remote ones, each sorted by name", () => {
    expect(labels("")).toEqual([
      "feature-2",
      "Feature-10",
      "feature/login",
      "main",
      "origin/feature/login",
      "origin/main",
      "upstream/release",
    ]);
  });

  it("matches every word, in any order and case", () => {
    expect(labels("LOG fea")).toEqual(["feature/login", "origin/feature/login"]);
    expect(labels("origin main")).toEqual(["origin/main"]);
    expect(labels("nothing")).toEqual([]);
  });

  it("keeps the current branch and the ahead / behind / gone detail", () => {
    const main = filterBranches(list, "main")[0];
    expect(main).toMatchObject({ label: "main", current: true, kind: "local", detail: "↑1 ↓2" });
    expect(filterBranches(list, "feature-2")[0].detail).toBe("gone");
    expect(filterBranches(list, "release")[0]).toMatchObject({ kind: "remote", fullName: "refs/remotes/upstream/release" });
  });
});
