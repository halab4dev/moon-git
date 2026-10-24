import { describe, expect, it } from "vitest";
import { findConflicts, resolvedLines, sideView } from "./conflictBlocks";

const file = [
  "top", //            0
  "<<<<<<< HEAD", //   1
  "ours 1", //         2
  "ours 2", //         3
  "=======", //        4
  "theirs", //         5
  ">>>>>>> feature", // 6
  "middle", //         7
  "<<<<<<< HEAD", //   8
  "o", //              9
  "||||||| base", //   10
  "b", //              11
  "=======", //        12
  "t", //              13
  ">>>>>>> feature", // 14
  "bottom", //         15
];

describe("findConflicts", () => {
  it("finds blocks with and without a base section", () => {
    expect(findConflicts(file)).toEqual([
      { start: 1, base: null, middle: 4, end: 6 },
      { start: 8, base: 10, middle: 12, end: 14 },
    ]);
  });

  it("skips broken blocks and look-alike lines", () => {
    expect(findConflicts(["<<<<<<< HEAD", "x", ">>>>>>> y"])).toEqual([]);
    // A Markdown heading underline is not a block.
    expect(findConflicts(["Title", "=======", "text"])).toEqual([]);
    // "<<<<<<<<" (8 characters) is not a marker.
    expect(findConflicts(["<<<<<<<< x", "=======", ">>>>>>> y"])).toEqual([]);
  });
});

describe("resolvedLines", () => {
  const [first, second] = findConflicts(file);

  it("takes one side, or both", () => {
    expect(resolvedLines(file, first, "ours")).toEqual(["ours 1", "ours 2"]);
    expect(resolvedLines(file, first, "theirs")).toEqual(["theirs"]);
    expect(resolvedLines(file, first, "both")).toEqual(["ours 1", "ours 2", "theirs"]);
    // The base section is never part of the result.
    expect(resolvedLines(file, second, "ours")).toEqual(["o"]);
  });
});

describe("sideView", () => {
  it("replaces every block by one side and tells where blocks start", () => {
    const blocks = findConflicts(file);
    expect(sideView(file, blocks, "theirs")).toEqual({
      lines: ["top", "theirs", "middle", "t", "bottom"],
      blockStarts: [1, 3],
    });
  });
});
