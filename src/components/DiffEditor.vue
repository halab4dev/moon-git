<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import { currentTheme, languageForPath, monaco, onThemeChange } from "../monaco";

// Thin wrapper around Monaco's diff editor. Monaco computes the differences
// itself from the two texts.
const props = defineProps<{
  path: string;
  oldText: string;
  newText: string;
  sideBySide: boolean;
  ignoreWhitespace: boolean;
}>();

const container = ref<HTMLDivElement | null>(null);

// Monaco objects are not UI state: keep them in plain variables, not ref().
// Vue would otherwise wrap them in reactive proxies, which Monaco doesn't expect.
let editor: monaco.editor.IStandaloneDiffEditor | null = null;
let originalModel: monaco.editor.ITextModel | null = null;
let modifiedModel: monaco.editor.ITextModel | null = null;
let shownPath = "";
let stopThemeListener: (() => void) | null = null;

onMounted(() => {
  if (container.value === null) {
    return;
  }
  editor = monaco.editor.createDiffEditor(container.value, {
    readOnly: true,
    originalEditable: false,
    renderSideBySide: props.sideBySide,
    ignoreTrimWhitespace: props.ignoreWhitespace,
    // Resize with the panel (window resize, splitter drag).
    automaticLayout: true,
    minimap: { enabled: false },
    // Always the whole file, unchanged lines included (Monaco can fold them away).
    hideUnchangedRegions: { enabled: false },
    scrollBeyondLastLine: false,
    fontSize: 12,
    theme: currentTheme(),
  });
  showTexts();
  stopThemeListener = onThemeChange(() => monaco.editor.setTheme(currentTheme()));
});

function showTexts() {
  if (editor === null) {
    return;
  }
  // Same file reloaded (e.g. after a refresh): keep the scroll position.
  const viewState = props.path === shownPath ? editor.saveViewState() : null;

  const language = languageForPath(props.path);
  const newOriginal = monaco.editor.createModel(props.oldText, language);
  const newModified = monaco.editor.createModel(props.newText, language);
  editor.setModel({ original: newOriginal, modified: newModified });

  // Models are not freed automatically; dispose the previous pair.
  originalModel?.dispose();
  modifiedModel?.dispose();
  originalModel = newOriginal;
  modifiedModel = newModified;
  shownPath = props.path;

  if (viewState !== null) {
    editor.restoreViewState(viewState);
  }
}

watch(() => [props.path, props.oldText, props.newText], showTexts);

watch(
  () => [props.sideBySide, props.ignoreWhitespace],
  () => editor?.updateOptions({ renderSideBySide: props.sideBySide, ignoreTrimWhitespace: props.ignoreWhitespace }),
);

onBeforeUnmount(() => {
  stopThemeListener?.();
  editor?.dispose();
  originalModel?.dispose();
  modifiedModel?.dispose();
});
</script>

<template>
  <div ref="container" class="diff-editor"></div>
</template>

<style scoped>
.diff-editor {
  width: 100%;
  height: 100%;
}
</style>
