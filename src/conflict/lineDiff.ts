// Line-by-line comparison of two texts, used by the conflict solver to
// highlight what each side changed and to find the matching place in the
// other columns.
//
// The algorithm is Myers' (1986), the one `git diff` uses by default. It finds
// the fewest lines to delete from `a` and insert from `b`. Picture a grid with
// `a` along the top and `b` down the side: moving right deletes a line of `a`,
// moving down inserts a line of `b`, and moving diagonally keeps a line both
// have. Myers explores, for d = 0, 1, 2…, how far each diagonal k = x - y can
// get with exactly d deletions/insertions, and stops at the first d that
// reaches the bottom-right corner. Then it walks back to recover the path.
//
// Plain TypeScript, no Vue, tested in lineDiff.test.ts.

/** A changed region: lines a[aStart..aEnd) were replaced by b[bStart..bEnd). 0-based, end exclusive. */
export interface Hunk {
  aStart: number;
  aEnd: number;
  bStart: number;
  bEnd: number;
}

/**
 * The changed regions between `a` and `b`, in order.
 * `maxEdits`: beyond that many changed lines, give up on precision and
 * report the whole differing middle as one hunk (keeps huge rewrites fast).
 */
export function diffLines(a: string[], b: string[], maxEdits = 4000): Hunk[] {
  // Most edits touch a small part of a file: skip the equal start and end first.
  let prefix = 0;
  while (prefix < a.length && prefix < b.length && a[prefix] === b[prefix]) {
    prefix++;
  }
  let suffix = 0;
  while (
    suffix < a.length - prefix &&
    suffix < b.length - prefix &&
    a[a.length - 1 - suffix] === b[b.length - 1 - suffix]
  ) {
    suffix++;
  }
  const midA = a.slice(prefix, a.length - suffix);
  const midB = b.slice(prefix, b.length - suffix);
  if (midA.length === 0 && midB.length === 0) {
    return [];
  }

  const matches = myersMatches(midA, midB, maxEdits);
  const hunks: Hunk[] = [];
  if (matches === null) {
    hunks.push({ aStart: 0, aEnd: midA.length, bStart: 0, bEnd: midB.length });
  } else {
    // A hunk is every gap between two kept (matching) lines.
    let nextA = 0;
    let nextB = 0;
    for (const [x, y] of [...matches, [midA.length, midB.length] as [number, number]]) {
      if (x > nextA || y > nextB) {
        hunks.push({ aStart: nextA, aEnd: x, bStart: nextB, bEnd: y });
      }
      nextA = x + 1;
      nextB = y + 1;
    }
  }
  // Back to positions in the full texts.
  return hunks.map((h) => ({
    aStart: h.aStart + prefix,
    aEnd: h.aEnd + prefix,
    bStart: h.bStart + prefix,
    bEnd: h.bEnd + prefix,
  }));
}

/**
 * Where line `line` of `a` ended up in `b`. A line inside a changed region
 * maps to the start of that region in `b`.
 */
export function mapLine(hunks: Hunk[], line: number): number {
  let shift = 0;
  for (const h of hunks) {
    if (line < h.aStart) {
      break;
    }
    if (line < h.aEnd) {
      return h.bStart;
    }
    shift = h.bEnd - h.aEnd;
  }
  return line + shift;
}

// The pairs [x, y] with a[x] === b[y] kept by the shortest edit, in order;
// null when more than `maxEdits` edits are needed.
function myersMatches(a: string[], b: string[], maxEdits: number): [number, number][] | null {
  const n = a.length;
  const m = b.length;
  const max = n + m;
  const offset = max + 1; // v[k + offset]: diagonals k go from -max to max
  // v[k]: the furthest x reached on diagonal k so far.
  const v = new Int32Array(2 * max + 3);
  // A copy of v at the start of each round d, to walk back afterwards.
  const trace: Int32Array[] = [];

  for (let d = 0; d <= Math.min(max, maxEdits); d++) {
    trace.push(v.slice());
    for (let k = -d; k <= d; k += 2) {
      // Come from diagonal k+1 (a step down = insert) or k-1 (a step right
      // = delete), whichever got further.
      const down = k === -d || (k !== d && v[k - 1 + offset] < v[k + 1 + offset]);
      let x = down ? v[k + 1 + offset] : v[k - 1 + offset] + 1;
      let y = x - k;
      // Follow equal lines diagonally for free (a "snake").
      while (x < n && y < m && a[x] === b[y]) {
        x++;
        y++;
      }
      v[k + offset] = x;
      if (x >= n && y >= m) {
        return walkBack(trace, a, b, offset);
      }
    }
  }
  return null;
}

// From the end back to the start: in each round d, find which diagonal we
// came from, and collect the equal lines of the snake on the way.
function walkBack(trace: Int32Array[], a: string[], b: string[], offset: number): [number, number][] {
  const matches: [number, number][] = [];
  let x = a.length;
  let y = b.length;
  for (let d = trace.length - 1; d >= 0; d--) {
    const v = trace[d];
    const k = x - y;
    const down = k === -d || (k !== d && v[k - 1 + offset] < v[k + 1 + offset]);
    const prevK = down ? k + 1 : k - 1;
    const prevX = d === 0 ? 0 : v[prevK + offset];
    const prevY = d === 0 ? 0 : prevX - prevK;
    // The snake: diagonal steps back to where the d-th edit ended.
    while (x > prevX && y > prevY) {
      x--;
      y--;
      matches.push([x, y]);
    }
    x = prevX;
    y = prevY;
  }
  return matches.reverse();
}
