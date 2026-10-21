import { describe, expect, it } from "vitest";
import { LaneLayout, layoutGraph, type GraphCommit, type GraphLayout, type GraphLine } from "./laneLayout";

// Builds commits from "id:parent1,parent2" strings, newest first.
function history(...specs: string[]): GraphCommit[] {
  return specs.map((spec) => {
    const [id, parents = ""] = spec.split(":");
    return { id, parents: parents ? parents.split(",") : [] };
  });
}

function lanesOf(layout: GraphLayout): number[] {
  return layout.rows.map((row) => row.lane);
}

// Columns where lines leave a row at the bottom edge / enter it at the top edge.
function bottomEdge(lines: GraphLine[]): Set<number> {
  return new Set(lines.filter((l) => l.kind !== "top").map((l) => l.to));
}
function topEdge(lines: GraphLine[]): Set<number> {
  return new Set(lines.filter((l) => l.kind !== "bottom").map((l) => l.from));
}

/**
 * Every line leaving a row at the bottom must be continued by the row below,
 * and nothing may start out of nowhere at the top. Checked for every layout
 * in these tests, so a broken line in any scenario is caught.
 */
function expectConnected(layout: GraphLayout) {
  const rows = layout.rows;
  expect(topEdge(rows[0]?.lines ?? [])).toEqual(new Set());
  for (let i = 0; i + 1 < rows.length; i++) {
    expect(topEdge(rows[i + 1].lines), `between rows ${i} and ${i + 1}`).toEqual(bottomEdge(rows[i].lines));
  }
}

describe("layoutGraph", () => {
  it("returns nothing for an empty history", () => {
    expect(layoutGraph([])).toEqual({ rows: [], maxLanes: 0 });
  });

  it("keeps a straight history in one lane", () => {
    const layout = layoutGraph(history("c:b", "b:a", "a"));

    expect(lanesOf(layout)).toEqual([0, 0, 0]);
    expect(layout.maxLanes).toBe(1);
    // Tip: only a line down. Middle: in and out. Root: only a line in.
    expect(layout.rows[0].lines.map((l) => l.kind)).toEqual(["bottom"]);
    expect(layout.rows[1].lines.map((l) => l.kind)).toEqual(["top", "bottom"]);
    expect(layout.rows[2].lines.map((l) => l.kind)).toEqual(["top"]);
    expectConnected(layout);
  });

  it("opens a lane for a merged branch and closes it at the fork point", () => {
    //  m       merge of a (main) and b (side)
    //  | \
    //  a  |
    //  |  b
    //  | /
    //  base
    const layout = layoutGraph(history("m:a,b", "a:base", "b:base", "base"));

    expect(lanesOf(layout)).toEqual([0, 0, 1, 0]);
    expect(layout.maxLanes).toBe(2);
    expect(layout.rows[0].lines).toContainEqual(expect.objectContaining({ from: 0, to: 1, kind: "bottom" }));
    // The side lane runs past a, then ends in base.
    expect(layout.rows[1].lines).toContainEqual(expect.objectContaining({ from: 1, to: 1, kind: "full" }));
    expect(layout.rows[3].lines).toContainEqual(expect.objectContaining({ from: 1, to: 0, kind: "top" }));
    expectConnected(layout);
  });

  it("gives each branch its own color, and a lane keeps its color", () => {
    const layout = layoutGraph(history("m:a,b", "a:base", "b:base", "base"));
    const [m, a, b, base] = layout.rows;

    expect(a.color).toBe(m.color);
    expect(base.color).toBe(m.color);
    expect(b.color).not.toBe(m.color);
    // The merge line has the color of the branch it comes from.
    const mergeLine = m.lines.find((l) => l.to === 1)!;
    expect(mergeLine.color).toBe(b.color);
  });

  it("puts unmerged branch tips side by side", () => {
    // feature and main both start from base; nothing merged.
    const layout = layoutGraph(history("feature:f1", "main:base", "f1:base", "base"));

    expect(lanesOf(layout)).toEqual([0, 1, 0, 0]);
    expect(layout.rows[1].color).not.toBe(layout.rows[0].color);
    // At base, both lanes join.
    expect(layout.rows[3].lines.filter((l) => l.kind === "top")).toHaveLength(2);
    expectConnected(layout);
  });

  it("reuses a freed column for the next branch tip", () => {
    //  m2          y forks from m1 and ends there; column 1 is then free,
    //  | \         so the lane of x (merged by m1) reuses it instead of
    //  |  y        opening a third column.
    //  | /
    //  m1
    //  | \
    //  |  x
    //  | /
    //  a
    const layout = layoutGraph(history("m2:m1,y", "y:m1", "m1:a,x", "x:a", "a"));

    expect(lanesOf(layout)).toEqual([0, 1, 0, 1, 0]);
    expect(layout.maxLanes).toBe(2);
    expectConnected(layout);
  });

  it("joins a merge line to a lane already heading to that parent", () => {
    // Two merges of the same side commit s: the second one must not open a third lane.
    const layout = layoutGraph(history("m2:m1,s", "m1:a,s", "a:base", "s:base", "base"));

    expect(layout.maxLanes).toBe(2);
    const merges = layout.rows[1].lines.filter((l) => l.kind === "bottom" && l.from !== l.to);
    expect(merges).toEqual([expect.objectContaining({ from: 0, to: 1 })]);
    expectConnected(layout);
  });

  it("handles an octopus merge with three parents", () => {
    const layout = layoutGraph(history("o:a,b,c", "a:base", "b:base", "c:base", "base"));

    expect(lanesOf(layout)).toEqual([0, 0, 1, 2, 0]);
    expect(layout.maxLanes).toBe(3);
    expectConnected(layout);
  });

  it("keeps a lane open when the parent is not loaded yet", () => {
    // Only the first page is loaded: the line continues below the last row.
    const layout = layoutGraph(history("c:b", "b:a"));
    const last = layout.rows[1];
    expect(last.lines).toContainEqual(expect.objectContaining({ from: 0, to: 0, kind: "bottom" }));
  });

  it("gives the same result when commits are added page by page", () => {
    const commits = generatedHistory();
    const paged = new LaneLayout();
    for (let start = 0; start < commits.length; start += 37) {
      paged.add(commits.slice(start, start + 37));
    }
    expect(paged.rows).toEqual(layoutGraph(commits).rows);
    expect(paged.maxLanes).toBe(layoutGraph(commits).maxLanes);
  });

  it("stays connected on a larger generated history", () => {
    const layout = layoutGraph(generatedHistory());
    expect(layout.rows).toHaveLength(400);
    expectConnected(layout);
  });
});

/** A deterministic pseudo-random history of 400 commits: branches forking and merging. */
function generatedHistory(): GraphCommit[] {
  const commits: GraphCommit[] = [];
  let seed = 42;
  const random = () => (seed = (seed * 1103515245 + 12345) % 2 ** 31) / 2 ** 31;
  // Build oldest -> newest, then reverse to newest first.
  const ids: string[] = [];
  for (let i = 0; i < 400; i++) {
    const parents: string[] = [];
    if (i > 0) parents.push(ids[i - 1 - Math.floor(random() * Math.min(i, 5))]);
    if (i > 5 && random() < 0.2) parents.push(ids[i - 2 - Math.floor(random() * 4)]);
    ids.push(`c${i}`);
    commits.push({ id: `c${i}`, parents: [...new Set(parents)] });
  }
  commits.reverse();
  return commits;
}
