// Loads only the parts of Monaco (the VS Code editor) that the diff view needs:
// the editor core, which includes the diff editor, and syntax coloring for
// ~80 languages (each language's colors load on first use).
// Language services (IntelliSense for TypeScript, JSON, CSS...) are left out:
// they are large and a read-only diff doesn't need them.
import * as monaco from "monaco-editor/editor/editor.api";
import "monaco-editor/basic-languages/monaco.contribution";
// "?worker" is a Vite feature: bundle this file as a web worker script.
import EditorWorker from "monaco-editor/editor/editor.worker?worker";

// Monaco computes diffs in a web worker (a background thread of the webview),
// so comparing big files does not freeze the UI.
self.MonacoEnvironment = {
  getWorker: () => new EditorWorker(),
};

export { monaco };

// Monaco has no Vue language; HTML coloring fits a .vue file best.
const EXTRA_EXTENSIONS: Record<string, string> = {
  ".vue": "html",
};

/** Monaco language id for a file path, e.g. "src/main.rs" -> "rust". */
export function languageForPath(path: string): string {
  const name = path.slice(path.lastIndexOf("/") + 1).toLowerCase();
  const dot = name.lastIndexOf(".");
  const extension = dot === -1 ? "" : name.slice(dot);
  if (EXTRA_EXTENSIONS[extension]) {
    return EXTRA_EXTENSIONS[extension];
  }
  for (const language of monaco.languages.getLanguages()) {
    // Some languages are known by full file name, e.g. "Dockerfile".
    if (language.filenames?.some((f) => f.toLowerCase() === name)) {
      return language.id;
    }
    if (extension && language.extensions?.some((ext) => ext.toLowerCase() === extension)) {
      return language.id;
    }
  }
  return "plaintext";
}

const darkQuery = window.matchMedia("(prefers-color-scheme: dark)");

export function currentTheme(): string {
  return darkQuery.matches ? "vs-dark" : "vs";
}

/** Calls `listener` when the OS switches between light and dark mode. Returns an unsubscribe function. */
export function onThemeChange(listener: () => void): () => void {
  darkQuery.addEventListener("change", listener);
  return () => darkQuery.removeEventListener("change", listener);
}
