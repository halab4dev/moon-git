import { describe, expect, it } from "vitest";
import { diffLines, mapLine } from "./lineDiff";

const lines = (text: string) => (text === "" ? [] : text.split(" "));

describe("diffLines", () => {
  it("finds nothing in equal texts", () => {
    expect(diffLines(lines("a b c"), lines("a b c"))).toEqual([]);
  });

  it("finds a changed line in the middle", () => {
    expect(diffLines(lines("a b c"), lines("a X c"))).toEqual([{ aStart: 1, aEnd: 2, bStart: 1, bEnd: 2 }]);
  });

  it("finds insertions and deletions", () => {
    // "b" deleted, "Y" inserted after "d".
    expect(diffLines(lines("a b c d e"), lines("a c d Y e"))).toEqual([
      { aStart: 1, aEnd: 2, bStart: 1, bEnd: 1 },
      { aStart: 4, aEnd: 4, bStart: 3, bEnd: 4 },
    ]);
  });

  it("handles empty texts", () => {
    expect(diffLines([], lines("a b"))).toEqual([{ aStart: 0, aEnd: 0, bStart: 0, bEnd: 2 }]);
    expect(diffLines(lines("a b"), [])).toEqual([{ aStart: 0, aEnd: 2, bStart: 0, bEnd: 0 }]);
  });

  it("keeps the longest common lines (the classic Myers example)", () => {
    // ABCABBA -> CBABAC: 5 edits, 4 kept lines.
    const a = "A B C A B B A".split(" ");
    const b = "C B A B A C".split(" ");
    const hunks = diffLines(a, b);
    const changed = hunks.reduce((sum, h) => sum + (h.aEnd - h.aStart) + (h.bEnd - h.bStart), 0);
    expect(changed).toBe(5);
  });

  it("gives up on precision beyond maxEdits", () => {
    expect(diffLines(lines("a b c d"), lines("w x y z"), 2)).toEqual([{ aStart: 0, aEnd: 4, bStart: 0, bEnd: 4 }]);
  });
});

describe("mapLine", () => {
  // a: 0 1 2 3 4 ; b: line 1 replaced by two lines, line 3 deleted.
  const hunks = diffLines(lines("a b c d e"), lines("a X Y c e"));

  it("maps lines before, inside and after changes", () => {
    expect(mapLine(hunks, 0)).toBe(0); // a
    expect(mapLine(hunks, 1)).toBe(1); // b -> start of X Y
    expect(mapLine(hunks, 2)).toBe(3); // c
    expect(mapLine(hunks, 3)).toBe(4); // d (deleted) -> where it was
    expect(mapLine(hunks, 4)).toBe(4); // e
  });
});
