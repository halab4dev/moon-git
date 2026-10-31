import { describe, expect, it } from "vitest";
import { buildTree, rangeBetween, visibleRepoOrder } from "./repoTree";
import type { RepoEntry, RepoGroup } from "../types/repo";

const group = (id: string, name: string, parentId: string | null = null): RepoGroup => ({ id, name, parentId });
const repo = (id: string, groupId: string | null = null): RepoEntry => ({ id, name: id, path: `/${id}`, groupId });

// Work (a, b) with subgroup Team (c); Other (d); top level e, f.
const tree = buildTree(
  [group("w", "Work"), group("t", "Team", "w"), group("o", "Other")],
  [repo("a", "w"), repo("b", "w"), repo("c", "t"), repo("d", "o"), repo("e"), repo("f")],
);

describe("visibleRepoOrder", () => {
  it("lists repositories as the sidebar shows them", () => {
    expect(visibleRepoOrder(tree, () => false)).toEqual(["d", "c", "a", "b", "e", "f"]);
  });

  it("skips the inside of collapsed groups", () => {
    expect(visibleRepoOrder(tree, (id) => id === "w")).toEqual(["d", "e", "f"]);
    expect(visibleRepoOrder(tree, (id) => id === "t")).toEqual(["d", "a", "b", "e", "f"]);
  });
});

describe("rangeBetween", () => {
  const order = ["d", "c", "a", "b", "e", "f"];

  it("takes everything between, in either direction", () => {
    expect(rangeBetween(order, "c", "b")).toEqual(["c", "a", "b"]);
    expect(rangeBetween(order, "e", "a")).toEqual(["a", "b", "e"]);
    expect(rangeBetween(order, "a", "a")).toEqual(["a"]);
  });

  it("falls back to the clicked one when the start isn't visible", () => {
    expect(rangeBetween(order, null, "b")).toEqual(["b"]);
    expect(rangeBetween(order, "hidden", "b")).toEqual(["b"]);
    expect(rangeBetween(order, "a", "hidden")).toEqual([]);
  });
});
