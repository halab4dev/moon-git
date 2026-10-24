<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, shallowRef } from "vue";
import { currentTheme, languageForPath, monaco, onThemeChange } from "../monaco";
import { findConflicts, resolvedLines, sideView, type BlockChoice, type ConflictBlock } from "../conflict/conflictBlocks";
import { diffLines, mapLine } from "../conflict/lineDiff";

// Three columns, like SmartGit's conflict solver:
//   left: our version (read-only) | middle: the result, editable | right: their version (read-only)
// The middle starts as the file git wrote, with conflict markers. Each
// conflict block can be replaced by the left part, the right part or both,
// or edited freely. The side columns highlight what that side changed
// compared with the common ancestor.
const props = defineProps<{
  path: string;
  base: string | null;
  ours: string;
  theirs: string;
  merged: string;
  oursLabel: string;
  theirsLabel: string;
  saving: boolean;
}>();

const emit = defineEmits<{
  save: [content: string, markResolved: boolean];
  dirty: [dirty: boolean];
}>();

const leftEl = ref<HTMLDivElement | null>(null);
const middleEl = ref<HTMLDivElement | null>(null);
const rightEl = ref<HTMLDivElement | null>(null);

// Monaco objects stay out of Vue's reactivity (as in DiffEditor.vue).
let left: monaco.editor.IStandaloneCodeEditor | null = null;
let middle: monaco.editor.IStandaloneCodeEditor | null = null;
let right: monaco.editor.IStandaloneCodeEditor | null = null;
let models: monaco.editor.ITextModel[] = [];
let middleDecorations: monaco.editor.IEditorDecorationsCollection | null = null;
let stopThemeListener: (() => void) | null = null;
let refreshTimer: ReturnType<typeof setTimeout> | undefined;

const blocks = shallowRef<ConflictBlock[]>([]);
const current = ref(0);
const dirty = ref(false);
// The content as last saved, to know whether there are unsaved edits.
let savedContent = props.merged;

const oursLines = props.ours.split(/\r?\n/);
const theirsLines = props.theirs.split(/\r?\n/);

const position = computed(() =>
  blocks.value.length === 0 ? "No conflicts left" : `Conflict ${current.value + 1} of ${blocks.value.length}`,
);

const OPTIONS: monaco.editor.IStandaloneEditorConstructionOptions = {
  automaticLayout: true,
  minimap: { enabled: false },
  scrollBeyondLastLine: false,
  fontSize: 12,
};

onMounted(() => {
  if (leftEl.value === null || middleEl.value === null || rightEl.value === null) {
    return;
  }
  const language = languageForPath(props.path);
  const leftModel = monaco.editor.createModel(props.ours, language);
  const middleModel = monaco.editor.createModel(props.merged, language);
  const rightModel = monaco.editor.createModel(props.theirs, language);
  models = [leftModel, middleModel, rightModel];
  const theme = currentTheme();
  left = monaco.editor.create(leftEl.value, { ...OPTIONS, model: leftModel, readOnly: true, theme });
  middle = monaco.editor.create(middleEl.value, { ...OPTIONS, model: middleModel, theme });
  right = monaco.editor.create(rightEl.value, { ...OPTIONS, model: rightModel, readOnly: true, theme });
  middleDecorations = middle.createDecorationsCollection();

  highlightSide(left, oursLines, "cs-ours-line");
  highlightSide(right, theirsLines, "cs-theirs-line");

  middle.onDidChangeModelContent(() => {
    dirty.value = middleModel.getValue() !== savedContent;
    emit("dirty", dirty.value);
    // Typing changes the blocks; re-read them shortly after, not on every key.
    clearTimeout(refreshTimer);
    refreshTimer = setTimeout(refreshBlocks, 150);
  });
  // Clicking into a block makes it the current one (for the Take buttons).
  middle.onDidChangeCursorPosition((event) => {
    const line = event.position.lineNumber - 1;
    const index = blocks.value.findIndex((b) => line >= b.start && line <= b.end);
    if (index !== -1) {
      current.value = index;
      decorateMiddle();
    }
  });
  // ⌘S / Ctrl+S saves, as in any editor.
  middle.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => save(false));

  refreshBlocks();
  goTo(0);
  stopThemeListener = onThemeChange(() => monaco.editor.setTheme(currentTheme()));
});

onBeforeUnmount(() => {
  clearTimeout(refreshTimer);
  stopThemeListener?.();
  for (const editor of [left, middle, right]) {
    editor?.dispose();
  }
  for (const model of models) {
    model.dispose();
  }
});

// ---- highlighting ----

// One whole-line decoration for lines from..to (0-based, inclusive).
function lines(from: number, to: number, className: string): monaco.editor.IModelDeltaDecoration[] {
  if (from > to) {
    return [];
  }
  return [{ range: new monaco.Range(from + 1, 1, to + 1, 1), options: { isWholeLine: true, className } }];
}

// In a side column: the lines that side changed since the common ancestor.
// (No ancestor when both sides added the file: nothing to compare with.)
function highlightSide(editor: monaco.editor.IStandaloneCodeEditor, sideLines: string[], className: string) {
  if (props.base === null) {
    return;
  }
  const hunks = diffLines(props.base.split(/\r?\n/), sideLines);
  editor.createDecorationsCollection(hunks.flatMap((h) => lines(h.bStart, h.bEnd - 1, className)));
}

function middleLines(): string[] {
  return middle?.getModel()?.getLinesContent() ?? [];
}

function refreshBlocks() {
  blocks.value = findConflicts(middleLines());
  current.value = Math.min(current.value, Math.max(0, blocks.value.length - 1));
  decorateMiddle();
}

// Marker lines, our part (blue, like the left column), the ancestor's part
// (grey, with diff3 style) and their part (green, like the right column).
function decorateMiddle() {
  const decorations = blocks.value.flatMap((b, i) => {
    const marker = i === current.value ? "cs-marker cs-current" : "cs-marker";
    const oursEnd = (b.base ?? b.middle) - 1;
    return [
      ...lines(b.start, b.start, marker),
      ...lines(b.start + 1, oursEnd, "cs-ours-line"),
      ...(b.base === null ? [] : [...lines(b.base, b.base, marker), ...lines(b.base + 1, b.middle - 1, "cs-base-line")]),
      ...lines(b.middle, b.middle, marker),
      ...lines(b.middle + 1, b.end - 1, "cs-theirs-line"),
      ...lines(b.end, b.end, marker),
    ];
  });
  middleDecorations?.set(decorations);
}

// ---- navigation ----

function goTo(index: number) {
  const block = blocks.value[index];
  if (block === undefined || middle === null) {
    return;
  }
  current.value = index;
  decorateMiddle();
  middle.revealLineInCenter(block.start + 1);
  middle.setPosition({ lineNumber: block.start + 2, column: 1 });
  // The side columns don't have the markers: find the matching line by
  // comparing "the result as that side sees it" with that side's file.
  const all = middleLines();
  for (const [editor, side, sideLines] of [
    [left, "ours", oursLines],
    [right, "theirs", theirsLines],
  ] as const) {
    const view = sideView(all, blocks.value, side);
    const line = mapLine(diffLines(view.lines, [...sideLines]), view.blockStarts[index]);
    editor?.revealLineInCenter(line + 1);
  }
}

function move(step: number) {
  if (blocks.value.length > 0) {
    goTo((current.value + step + blocks.value.length) % blocks.value.length);
  }
}

// ---- resolving a block ----

function take(choice: BlockChoice) {
  const model = middle?.getModel();
  const block = blocks.value[current.value];
  if (!middle || !model || block === undefined) {
    return;
  }
  const replacement = resolvedLines(model.getLinesContent(), block, choice);
  const startLine = block.start + 1;
  const endLine = block.end + 1;
  let range: monaco.Range;
  if (replacement.length > 0) {
    range = new monaco.Range(startLine, 1, endLine, model.getLineMaxColumn(endLine));
  } else if (endLine < model.getLineCount()) {
    // Nothing left: remove the lines including the line break after them.
    range = new monaco.Range(startLine, 1, endLine + 1, 1);
  } else if (startLine > 1) {
    // At the end of the file: remove the line break before them instead.
    range = new monaco.Range(startLine - 1, model.getLineMaxColumn(startLine - 1), endLine, model.getLineMaxColumn(endLine));
  } else {
    range = model.getFullModelRange();
  }
  // pushEditOperations: undoable with ⌘Z, like typing.
  middle.pushUndoStop();
  model.pushEditOperations([], [{ range, text: replacement.join(model.getEOL()) }], () => null);
  middle.pushUndoStop();
  refreshBlocks();
  goTo(Math.min(current.value, blocks.value.length - 1));
}

// ---- saving ----

function save(markResolved: boolean) {
  const content = middle?.getModel()?.getValue();
  if (content !== undefined && !props.saving) {
    emit("save", content, markResolved);
  }
}

/** Called by the window after a successful save. */
function markSaved(content: string) {
  savedContent = content;
  dirty.value = middle?.getModel()?.getValue() !== savedContent;
  emit("dirty", dirty.value);
}

defineExpose({ markSaved });
</script>

<template>
  <div class="solver">
    <div class="toolbar">
      <button :disabled="blocks.length === 0" title="Previous conflict" @click="move(-1)">‹</button>
      <span class="position">{{ position }}</span>
      <button :disabled="blocks.length === 0" title="Next conflict" @click="move(1)">›</button>
      <span class="separator"></span>
      <button :disabled="blocks.length === 0" :title="`Use the left part (${oursLabel}) for this conflict`" @click="take('ours')">
        Take Left
      </button>
      <button :disabled="blocks.length === 0" :title="`Use the right part (${theirsLabel}) for this conflict`" @click="take('theirs')">
        Take Right
      </button>
      <button :disabled="blocks.length === 0" title="Keep both parts: left, then right" @click="take('both')">Take Both</button>
      <span class="spacer"></span>
      <span v-if="dirty" class="muted">Unsaved changes</span>
      <button :disabled="saving" title="Save the result into the file (⌘S)" @click="save(false)">Save</button>
      <button class="primary" :disabled="saving" title="Save, stage the file, and close" @click="save(true)">
        Save &amp; Mark Resolved
      </button>
    </div>
    <div class="columns">
      <div class="column">
        <div class="title ours" :title="oursLabel">Ours: {{ oursLabel }}</div>
        <div ref="leftEl" class="editor"></div>
      </div>
      <div class="column">
        <div class="title">Result (editable)</div>
        <div ref="middleEl" class="editor"></div>
      </div>
      <div class="column">
        <div class="title theirs" :title="theirsLabel">Theirs: {{ theirsLabel }}</div>
        <div ref="rightEl" class="editor"></div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.solver {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 6px;
  padding-bottom: 6px;
}

.position {
  min-width: 110px;
  text-align: center;
}

.separator {
  width: 1px;
  align-self: stretch;
  background: var(--border);
}

.spacer {
  flex: 1;
}

.muted {
  color: var(--text-muted);
}

.columns {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 6px;
}

.column {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  border: 1px solid var(--border);
}

.title {
  padding: 3px 8px;
  border-bottom: 1px solid var(--border);
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.title.ours {
  color: var(--accent);
}

.title.theirs {
  color: var(--success);
}

.editor {
  flex: 1;
  min-height: 0;
}
</style>

<!-- Not scoped: Monaco renders these classes itself, outside this component's elements. -->
<style>
.cs-ours-line {
  background: rgba(59, 111, 216, 0.18);
}

.cs-theirs-line {
  background: rgba(26, 127, 55, 0.18);
}

.cs-base-line {
  background: rgba(128, 128, 128, 0.15);
}

.cs-marker {
  background: rgba(207, 34, 46, 0.15);
}

.cs-marker.cs-current {
  background: rgba(207, 34, 46, 0.35);
}
</style>
