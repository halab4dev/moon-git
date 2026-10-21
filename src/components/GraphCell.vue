<script setup lang="ts">
import type { GraphLine, GraphRow } from "../graph/laneLayout";

// Draws the graph part of one history row as a small SVG: the line pieces
// computed by laneLayout.ts, and the commit's dot on top of them.
const props = defineProps<{
  row: GraphRow;
  /** Merge commits get a hollow dot. */
  merge: boolean;
  /** Visible width in px; lanes beyond it are cut off. */
  width: number;
}>();

const LANE_WIDTH = 14; // px per lane, must match LANE_WIDTH in CommitList.vue
const HEIGHT = 24; // px, the row height
const MIDDLE = HEIGHT / 2;

// Readable on light and dark backgrounds.
const PALETTE = ["#3b82f6", "#16a34a", "#f59e0b", "#dc2626", "#9333ea", "#0d9488", "#db2777", "#65a30d"];

function x(lane: number): number {
  return lane * LANE_WIDTH + LANE_WIDTH / 2;
}

function colorOf(index: number): string {
  return PALETTE[index % PALETTE.length];
}

// SVG path for one piece. Pieces changing lanes are drawn as a soft S-curve
// (a cubic Bézier whose control points sit halfway down, above each end).
function path(line: GraphLine): string {
  const x1 = x(line.from);
  const x2 = x(line.to);
  const [y1, y2] = line.kind === "top" ? [0, MIDDLE] : line.kind === "bottom" ? [MIDDLE, HEIGHT] : [0, HEIGHT];
  if (x1 === x2) {
    return `M ${x1} ${y1} L ${x2} ${y2}`;
  }
  const yMid = (y1 + y2) / 2;
  return `M ${x1} ${y1} C ${x1} ${yMid}, ${x2} ${yMid}, ${x2} ${y2}`;
}
</script>

<template>
  <svg class="graph-cell" :width="width" :height="HEIGHT" :viewBox="`0 0 ${width} ${HEIGHT}`">
    <path
      v-for="(line, i) in props.row.lines"
      :key="i"
      :d="path(line)"
      :stroke="colorOf(line.color)"
      stroke-width="2"
      fill="none"
    />
    <circle
      :cx="x(row.lane)"
      :cy="MIDDLE"
      r="4"
      :fill="merge ? 'var(--bg)' : colorOf(row.color)"
      :stroke="colorOf(row.color)"
      stroke-width="2"
    />
  </svg>
</template>

<style scoped>
.graph-cell {
  display: block;
  overflow: hidden;
}
</style>
