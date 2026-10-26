<script setup lang="ts">
// Colored toolbar icons, drawn here as SVG (no icon library: we only need a
// handful). Each icon has its own color, and most have a light "duotone"
// fill behind the lines, like the toolbar icons of SmartGit or an IDE.
export type IconName =
  | "fetch"
  | "pull"
  | "push"
  | "refresh"
  | "stage"
  | "unstage"
  | "discard"
  | "stash"
  | "resolve"
  | "check"
  | "apply"
  | "pop"
  | "drop"
  | "folder"
  | "folderOpen"
  | "repo"
  | "commit";

defineProps<{
  name: IconName;
}>();

/**
 * One shape: `fill` = a soft area, `line` = drawn lines, `solid` = filled in
 * full color, `white` = white lines on a solid shape, `whiteSolid` = white
 * area on a solid shape, `danger` = a red badge (always red, whatever the
 * icon's color). `color`: this shape's own color instead of the icon's.
 */
interface Shape {
  d: string;
  kind: "fill" | "line" | "solid" | "white" | "whiteSolid" | "danger";
  color?: string;
}

interface Icon {
  /** Main color; the soft fill is the same color, mostly transparent. */
  color: string;
  shapes: Shape[];
}

const fill = (d: string): Shape => ({ d, kind: "fill" });
const line = (d: string): Shape => ({ d, kind: "line" });
const solid = (d: string): Shape => ({ d, kind: "solid" });
// White lines on top of a solid shape (e.g. the "+" in the green disc).
const white = (d: string): Shape => ({ d, kind: "white" });
const danger = (d: string): Shape => ({ d, kind: "danger" });
// A full-color area in its own color (two-tone icons like the folder).
const paint = (d: string, color: string): Shape => ({ d, kind: "solid", color });

/** A circle as a path (center, radius). */
function circle(cx: number, cy: number, r: number): string {
  return `M${cx} ${cy - r}a${r} ${r} 0 1 1 0 ${2 * r} ${r} ${r} 0 0 1 0-${2 * r}z`;
}

const CIRCLE = "M12 2.5a9.5 9.5 0 1 1 0 19 9.5 9.5 0 0 1 0-19z";
const TRAY = "M3.5 15v4.5a1 1 0 0 0 1 1h15a1 1 0 0 0 1-1V15";
const CLOUD = "M7 18H6.2a4.2 4.2 0 0 1-.7-8.35A6.2 6.2 0 0 1 17.4 8.3 4.85 4.85 0 0 1 17.8 18H17";

// The stash box, open at the bottom, with an arrow coming out of it.
const STASH_OUT: Shape[] = [
  fill("M5 6.5h14v5H5z"),
  line("M3 3h18v3.5H3z"),
  line("M5 6.5v5h2.5"),
  line("M12.5 11.5H19v-5"),
  line("M10 9.5V21"),
  line("M6.5 17.5L10 21l3.5-3.5"),
];
// A small disc at the bottom right, for the red ✕.
const BADGE = "M18.5 13.5a4.5 4.5 0 1 1 0 9 4.5 4.5 0 0 1 0-9z";

// On a 24×24 grid.
const ICONS: Record<IconName, Icon> = {
  // Download into the repository, files unchanged: blue cloud.
  fetch: { color: "#2f7de1", shapes: [fill(CLOUD + "z"), line(CLOUD), line("M12 11v10"), line("M8.8 17.8l3.2 3.2 3.2-3.2")] },
  // Down into the tray (green) / up out of it (orange).
  pull: { color: "#1f9d55", shapes: [fill("M3.5 15h17v5.5h-17z"), line(TRAY), line("M12 3v12"), line("M7.5 10.5L12 15l4.5-4.5")] },
  push: { color: "#e8590c", shapes: [fill("M3.5 15h17v5.5h-17z"), line(TRAY), line("M12 15V3"), line("M7.5 7.5L12 3l4.5 4.5")] },
  refresh: { color: "#0f8b8d", shapes: [line("M20 12a8 8 0 1 1-2.34-5.66"), line("M20 3.5V8h-4.5")] },
  // Discs with a white sign: + (green) / − (amber).
  stage: { color: "#2da44e", shapes: [solid(CIRCLE), white("M12 7.5v9"), white("M7.5 12h9")] },
  unstage: { color: "#d4880f", shapes: [solid(CIRCLE), white("M7.5 12h9")] },
  // "Undo": a red arrow turning back.
  discard: { color: "#d1242f", shapes: [line("M9 14L4 9l5-5"), line("M4 9h10a6 6 0 0 1 0 12h-3")] },
  // A purple storage box.
  stash: {
    color: "#8250df",
    shapes: [fill("M5 8h14v12H5z"), line("M3 4h18v4H3z"), line("M5 8v12h14V8"), line("M10 12h4")],
  },
  // A pencil: edit the file in the conflict solver.
  resolve: { color: "#bf8700", shapes: [fill("M4 20h4L19 9l-4-4L4 16z"), line("M4 20h4L19 9l-4-4L4 16z"), line("M13 7l4 4")] },
  check: { color: "#2da44e", shapes: [solid(CIRCLE), white("M7.5 12.5l3 3 6-6.5")] },
  // Taking a stash out: the same box as "stash", with the changes coming out
  // of it (arrow down). Apply and Drop adds a red ✕: the stash is deleted.
  apply: { color: "#8250df", shapes: STASH_OUT },
  pop: { color: "#8250df", shapes: [...STASH_OUT, danger(BADGE), white("M16.8 16.3l3.4 3.4"), white("M20.2 16.3l-3.4 3.4")] },
  drop: {
    color: "#d1242f",
    shapes: [
      fill("M6 7h12l-1 13H7z"),
      line("M4 7h16"),
      line("M6 7l1 13h10l1-13"),
      line("M9.5 7V4h5v3"),
      line("M10 11v5.5"),
      line("M14 11v5.5"),
    ],
  },
  // A commit as drawn in history graphs: a dot on a line.
  commit: { color: "#0969da", shapes: [fill(circle(12, 12, 4.5)), line(circle(12, 12, 4.5)), line("M2 12h5.5"), line("M16.5 12H22")] },
  // Sidebar. A group: a two-tone yellow folder, open while expanded.
  folder: {
    color: "#f0a202",
    shapes: [
      paint("M2 6a2 2 0 0 1 2-2h5l2 2h9a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2z", "#f0a202"),
      paint("M2 9.5h20V18a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2z", "#ffc83d"),
    ],
  },
  folderOpen: {
    color: "#f0a202",
    shapes: [
      paint("M2 6a2 2 0 0 1 2-2h5l2 2h8a2 2 0 0 1 2 2v2H5.2a1.5 1.5 0 0 0-1.43 1.05L2 17z", "#f0a202"),
      paint(
        "M5.2 10h17.1a1 1 0 0 1 .95 1.32l-2.4 7.32A2 2 0 0 1 18.95 20H3.3a1.3 1.3 0 0 1-1.24-1.7l2.2-7.25A1.5 1.5 0 0 1 5.2 10z",
        "#ffc83d",
      ),
    ],
  },
  // A repository: an orange tile (git's usual color) with a white branch.
  repo: {
    color: "#f05133",
    shapes: [
      solid("M5 2h14a3 3 0 0 1 3 3v14a3 3 0 0 1-3 3H5a3 3 0 0 1-3-3V5a3 3 0 0 1 3-3z"),
      white("M9 7v10"),
      white("M15 8.5v.5a3 3 0 0 1-3 3H9"),
      { d: circle(9, 6.5, 2) + circle(9, 17.5, 2) + circle(15, 7.5, 2), kind: "whiteSolid" },
    ],
  },
};
</script>

<template>
  <svg class="app-icon" viewBox="0 0 24 24" aria-hidden="true" :style="{ color: ICONS[name].color }">
    <path
      v-for="(shape, i) in ICONS[name].shapes"
      :key="i"
      :d="shape.d"
      :class="shape.kind"
      :style="shape.color ? { color: shape.color } : undefined"
    />
  </svg>
</template>

<style scoped>
.app-icon {
  width: 16px;
  height: 16px;
  margin-right: 5px;
  vertical-align: -3px;
  flex-shrink: 0;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.fill {
  fill: currentColor;
  fill-opacity: 0.18;
  stroke: none;
}

.line {
  fill: none;
  stroke: currentColor;
  stroke-width: 2;
}

.solid {
  fill: currentColor;
  stroke: none;
}

.white {
  fill: none;
  stroke: #fff;
  stroke-width: 2.2;
}

.whiteSolid {
  fill: #fff;
  stroke: none;
}

.danger {
  fill: #d1242f;
  stroke: none;
}
</style>
