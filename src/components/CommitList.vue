<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import GraphCell from "./GraphCell.vue";
import { formatCommitTime } from "./commitTime";
import type { GraphRow } from "../graph/laneLayout";
import type { CommitRef, CommitSummary } from "../types/log";

// The history as a virtualized list: only the rows inside the visible area
// (plus a few around it) exist in the DOM, so 50,000 commits cost about as
// much as 50. Every row has the same fixed height, which makes "which rows
// are visible" a simple division.
const props = defineProps<{
  commits: CommitSummary[];
  /** graph[i] is the graph part of commits[i]. */
  graph: GraphRow[];
  /** Widest graph row, in lanes. */
  graphLanes: number;
  /** Highlight filter: commits not in the set are dimmed. null = no filter. */
  matches: Set<string> | null;
  hasMore: boolean;
  loading: boolean;
}>();

const selected = defineModel<string | null>("selected", { required: true });

const emit = defineEmits<{
  /** The user scrolled close to the end of what is loaded. */
  loadMore: [];
  /** Right-click on a commit (it is selected first). */
  rowMenu: [commit: CommitSummary];
}>();

const ROW_HEIGHT = 24; // px, must match .row in the styles below
const OVERSCAN = 10; // extra rows above and below, so fast scrolling shows no gaps
const LOAD_AHEAD = 100; // ask for the next page this many rows before the end
const LANE_WIDTH = 14; // px, must match GraphCell.vue
// Very wide graphs (dozens of parallel branches) are cut off instead of
// pushing the messages out of sight.
const MAX_SHOWN_LANES = 20;

const graphWidth = computed(() => Math.max(1, Math.min(props.graphLanes, MAX_SHOWN_LANES)) * LANE_WIDTH);

const viewport = ref<HTMLDivElement | null>(null);
const scrollTop = ref(0);
const viewportHeight = ref(0);

const first = computed(() => Math.max(0, Math.floor(scrollTop.value / ROW_HEIGHT) - OVERSCAN));
const last = computed(() =>
  Math.min(props.commits.length, Math.ceil((scrollTop.value + viewportHeight.value) / ROW_HEIGHT) + OVERSCAN),
);
const visible = computed(() => props.commits.slice(first.value, last.value));
const totalHeight = computed(() => props.commits.length * ROW_HEIGHT);

function onScroll() {
  if (viewport.value) {
    scrollTop.value = viewport.value.scrollTop;
  }
}

// Load the next page in time, before the user reaches the end.
watch([last, () => props.commits.length, () => props.loading], () => {
  if (props.hasMore && !props.loading && last.value > props.commits.length - LOAD_AHEAD) {
    emit("loadMore");
  }
});

// The visible height changes with the window and the splitter.
let resizeObserver: ResizeObserver | null = null;
onMounted(() => {
  if (viewport.value) {
    viewportHeight.value = viewport.value.clientHeight;
    resizeObserver = new ResizeObserver(() => {
      viewportHeight.value = viewport.value?.clientHeight ?? 0;
    });
    resizeObserver.observe(viewport.value);
  }
});
onUnmounted(() => resizeObserver?.disconnect());

// ↑/↓ move the selection and keep it in view.
function move(step: number) {
  if (props.commits.length === 0) {
    return;
  }
  const current = props.commits.findIndex((c) => c.id === selected.value);
  const next = current === -1 ? 0 : Math.min(props.commits.length - 1, Math.max(0, current + step));
  selected.value = props.commits[next].id;
}

// Keep the selected commit visible, also when it was chosen from outside
// (e.g. "next match" in the filter). Already visible: nothing moves.
watch(selected, (id) => {
  const index = props.commits.findIndex((c) => c.id === id);
  const box = viewport.value;
  if (index === -1 || !box) {
    return;
  }
  const top = index * ROW_HEIGHT;
  if (top < box.scrollTop) {
    box.scrollTop = top;
  } else if (top + ROW_HEIGHT > box.scrollTop + box.clientHeight) {
    // Jumps from far away land in the middle; steps of one row just at the edge.
    const far = top > box.scrollTop + box.clientHeight + ROW_HEIGHT;
    box.scrollTop = far ? top - box.clientHeight / 2 : top + ROW_HEIGHT - box.clientHeight;
  }
});

function onRowMenu(commit: CommitSummary) {
  selected.value = commit.id;
  emit("rowMenu", commit);
}

// ---- formatting ----

function refClass(ref: CommitRef): string[] {
  return ["ref", ref.kind, ref.current ? "current" : ""];
}
</script>

<template>
  <div class="commit-list" :style="{ '--graph-width': `${graphWidth}px` }">
    <div class="header row">
      <span>Graph</span>
      <span>Message</span>
      <span>Author</span>
      <span>Date</span>
      <span>Commit</span>
    </div>
    <div
      ref="viewport"
      class="viewport"
      tabindex="0"
      @scroll="onScroll"
      @keydown.up.prevent="move(-1)"
      @keydown.down.prevent="move(1)"
    >
      <!-- Full-height spacer: gives the scrollbar the size of the whole list. -->
      <div class="spacer" :style="{ height: `${totalHeight}px` }">
        <div
          v-for="(commit, i) in visible"
          :key="commit.id"
          class="row commit"
          :class="{ selected: commit.id === selected, dim: matches !== null && !matches.has(commit.id) }"
          :style="{ top: `${(first + i) * ROW_HEIGHT}px` }"
          @click="selected = commit.id"
          @contextmenu.prevent="onRowMenu(commit)"
        >
          <GraphCell
            v-if="graph[first + i]"
            :row="graph[first + i]"
            :merge="commit.parents.length > 1"
            :width="graphWidth"
          />
          <span v-else></span>
          <span class="message" :title="commit.subject">
            <span v-for="r in commit.refs" :key="r.kind + r.name" :class="refClass(r)">{{ r.name }}</span>
            {{ commit.subject }}
          </span>
          <span class="author" :title="commit.authorEmail">{{ commit.authorName }}</span>
          <span class="date">{{ formatCommitTime(commit.authorTime) }}</span>
          <span class="hash">{{ commit.id.slice(0, 8) }}</span>
        </div>
      </div>
      <p v-if="!loading && commits.length === 0" class="muted empty">No commits to show.</p>
    </div>
  </div>
</template>

<style scoped>
.commit-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

/* Same columns for the header and every row. */
.row {
  display: grid;
  /* --graph-width is set from the widest graph row (see template). */
  grid-template-columns: var(--graph-width) minmax(0, 1fr) 160px 130px 80px;
  gap: 8px;
  align-items: center;
  height: 24px;
  padding: 0 8px;
  white-space: nowrap;
}

.row > span {
  overflow: hidden;
  text-overflow: ellipsis;
}

.header {
  flex-shrink: 0;
  border-bottom: 1px solid var(--border);
  font-weight: 600;
  color: var(--text-muted);
}

.viewport {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  position: relative;
}

.viewport:focus {
  outline: none;
}

.spacer {
  position: relative;
}

.commit {
  position: absolute;
  left: 0;
  right: 0;
  cursor: default;
  user-select: none;
}

/* Highlight filter: commits that don't match fade out, the graph stays. */
.commit.dim {
  opacity: 0.35;
}

.commit:hover {
  background: var(--bg-sidebar);
}

.commit.selected {
  background: var(--accent);
  color: #fff;
}

.commit.selected .author,
.commit.selected .date,
.commit.selected .hash {
  color: #fff;
}

.author,
.date,
.hash,
.muted {
  color: var(--text-muted);
}

.hash {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
}

.empty {
  padding: 8px;
}

/* Branch / tag labels in front of the message. */
.ref {
  display: inline-block;
  margin-right: 4px;
  padding: 0 5px;
  border: 1px solid currentColor;
  border-radius: 3px;
  font-size: 11px;
  line-height: 16px;
}

.ref.branch {
  color: var(--success);
}

.ref.remote {
  color: var(--accent);
}

.ref.tag {
  color: var(--warning);
}

.ref.head {
  color: var(--danger);
}

.ref.current {
  font-weight: 700;
}

.commit.selected .ref {
  color: #fff;
}
</style>
