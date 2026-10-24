// Finding git's conflict blocks in a file, and replacing one with a side.
//
//   <<<<<<< HEAD            ← start
//   our lines
//   ||||||| base            ← base (only with merge.conflictStyle = diff3)
//   ancestor's lines
//   =======                 ← middle
//   their lines
//   >>>>>>> feature         ← end
//
// Plain TypeScript, tested in conflictBlocks.test.ts.

/** Line indexes (0-based) of the marker lines of one block. */
export interface ConflictBlock {
  start: number;
  /** The "|||||||" line, or null without a base section. */
  base: number | null;
  middle: number;
  end: number;
}

export type BlockChoice = "ours" | "theirs" | "both";

// A marker is 7 equal characters, alone or followed by a space and a label.
function isMarker(line: string, char: string): boolean {
  const marker = char.repeat(7);
  return line === marker || line.startsWith(`${marker} `);
}

/** All complete conflict blocks, in order. Broken ones (missing a marker) are skipped. */
export function findConflicts(lines: string[]): ConflictBlock[] {
  const blocks: ConflictBlock[] = [];
  let start: number | null = null;
  let base: number | null = null;
  let middle: number | null = null;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (isMarker(line, "<")) {
      // A new start before the end: the previous block was broken.
      start = i;
      base = null;
      middle = null;
    } else if (start !== null && middle === null && isMarker(line, "|")) {
      base = i;
    } else if (start !== null && middle === null && line === "=======") {
      middle = i;
    } else if (start !== null && middle !== null && isMarker(line, ">")) {
      blocks.push({ start, base, middle, end: i });
      start = null;
      base = null;
      middle = null;
    }
  }
  return blocks;
}

/** The lines of each part of a block. */
export function blockParts(lines: string[], block: ConflictBlock) {
  return {
    ours: lines.slice(block.start + 1, block.base ?? block.middle),
    base: block.base === null ? [] : lines.slice(block.base + 1, block.middle),
    theirs: lines.slice(block.middle + 1, block.end),
  };
}

/** What a block becomes when resolved with `choice` (the marker lines go away). */
export function resolvedLines(lines: string[], block: ConflictBlock, choice: BlockChoice): string[] {
  const { ours, theirs } = blockParts(lines, block);
  switch (choice) {
    case "ours":
      return ours;
    case "theirs":
      return theirs;
    case "both":
      return [...ours, ...theirs];
  }
}

/**
 * The file as one side sees it: every block replaced by that side's lines,
 * plus where each block starts in the result. Used to find the matching
 * place in the ours / theirs column.
 */
export function sideView(
  lines: string[],
  blocks: ConflictBlock[],
  side: "ours" | "theirs",
): { lines: string[]; blockStarts: number[] } {
  const result: string[] = [];
  const blockStarts: number[] = [];
  let next = 0;
  for (const block of blocks) {
    result.push(...lines.slice(next, block.start));
    blockStarts.push(result.length);
    result.push(...resolvedLines(lines, block, side));
    next = block.end + 1;
  }
  result.push(...lines.slice(next));
  return { lines: result, blockStarts };
}
