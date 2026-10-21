// Lane layout for the commit graph: decides in which column ("lane") each
// commit's dot is drawn, and which line pieces each row contains.
//
// Input: commits newest first, every commit before its parents (git log
// --date-order guarantees this). Output: one GraphRow per commit.
//
// The idea: walk down the list keeping a list of open lanes. An open lane
// "waits" for one commit id (the parent that the line above is heading to).
//   1. A commit takes the lane that waits for it. Several lanes may wait for
//      the same commit (branches joining): they all end in its dot, and the
//      leftmost one is kept. If no lane waits for it, the commit is the tip
//      of a branch and gets a free lane.
//   2. Below the dot, the lane continues towards the first parent.
//   3. Every further parent (a merge) is joined to the lane already waiting
//      for it, or gets a new lane.
// Lanes never move sideways: a line that passes through a row stays in its
// column, and a freed column is reused by the next branch tip.
//
// Pure TypeScript without Vue, so it can be tested on its own (laneLayout.test.ts).

/** The part of a commit the layout needs. */
export interface GraphCommit {
  id: string;
  parents: readonly string[];
}

/**
 * A line piece inside one row, in lane units. A row is split at its dot:
 * - "top":    from lane `from` at the top edge to the dot (lane `to`), the middle
 * - "bottom": from the dot (lane `from`) to lane `to` at the bottom edge
 * - "full":   straight through the row in lane `from` (= `to`), not touching the dot
 */
export interface GraphLine {
  from: number;
  to: number;
  kind: "top" | "bottom" | "full";
  /** Color index; the renderer maps it onto its palette. */
  color: number;
}

export interface GraphRow {
  /** Column of the commit's dot. */
  lane: number;
  color: number;
  lines: GraphLine[];
}

export interface GraphLayout {
  rows: GraphRow[];
  /** Widest row, to size the graph column. */
  maxLanes: number;
}

interface OpenLane {
  /** Id of the commit this lane is heading to. */
  waitsFor: string;
  color: number;
}

/**
 * Builds the layout page by page: `add()` continues where the previous call
 * stopped, so loading 500 more commits only lays out those 500 instead of
 * everything again. (Like a Java class with private state.)
 */
export class LaneLayout {
  /** One row per commit added so far. */
  readonly rows: GraphRow[] = [];
  /** Widest row so far, to size the graph column. */
  maxLanes = 0;

  // Index = column. null = free column (may be reused by a new branch).
  private lanes: (OpenLane | null)[] = [];
  private nextColor = 0;

  /** Lays out the next commits, which must come right after the ones added before. */
  add(commits: readonly GraphCommit[]): void {
    for (const commit of commits) {
      this.addOne(commit);
    }
  }

  private addOne(commit: GraphCommit): void {
    const lines: GraphLine[] = [];

    // 1. Which open lanes end in this commit?
    const incoming: number[] = [];
    this.lanes.forEach((open, column) => {
      if (open?.waitsFor === commit.id) {
        incoming.push(column);
      }
    });

    let lane: number;
    let color: number;
    if (incoming.length > 0) {
      lane = incoming[0];
      color = this.lanes[lane]!.color;
    } else {
      lane = freeColumn(this.lanes); // tip of a branch
      color = this.nextColor++;
    }

    // Top half: lines into the dot, and every other open lane passing by.
    this.lanes.forEach((open, column) => {
      if (open === null) {
        return;
      }
      if (open.waitsFor === commit.id) {
        lines.push({ from: column, to: lane, kind: "top", color: open.color });
      } else {
        lines.push({ from: column, to: column, kind: "full", color: open.color });
      }
    });
    for (const column of incoming) {
      this.lanes[column] = null;
    }

    // 2. Below the dot: on towards the first parent, in the same lane and color.
    const [firstParent, ...otherParents] = commit.parents;
    if (firstParent !== undefined) {
      this.lanes[lane] = { waitsFor: firstParent, color };
      lines.push({ from: lane, to: lane, kind: "bottom", color });
    }

    // 3. Merge parents: join the lane already heading there, or open a new one.
    for (const parent of otherParents) {
      let target = this.lanes.findIndex((open) => open?.waitsFor === parent);
      if (target === -1) {
        target = freeColumn(this.lanes);
        this.lanes[target] = { waitsFor: parent, color: this.nextColor++ };
      }
      lines.push({ from: lane, to: target, kind: "bottom", color: this.lanes[target]!.color });
    }

    // Free columns at the right end are not needed any more.
    while (this.lanes.length > 0 && this.lanes[this.lanes.length - 1] === null) {
      this.lanes.pop();
    }

    this.rows.push({ lane, color, lines });
    this.maxLanes = Math.max(this.maxLanes, widthOf(lane, lines));
  }
}

/** Lays out a whole list at once. */
export function layoutGraph(commits: readonly GraphCommit[]): GraphLayout {
  const layout = new LaneLayout();
  layout.add(commits);
  return { rows: layout.rows, maxLanes: layout.maxLanes };
}

/** First free column, or a new one at the right end. */
function freeColumn(lanes: (OpenLane | null)[]): number {
  const free = lanes.indexOf(null);
  if (free !== -1) {
    return free;
  }
  lanes.push(null);
  return lanes.length - 1;
}

function widthOf(lane: number, lines: GraphLine[]): number {
  let width = lane + 1;
  for (const line of lines) {
    width = Math.max(width, line.from + 1, line.to + 1);
  }
  return width;
}
