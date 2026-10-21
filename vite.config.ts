import { defineConfig, transformWithOxc, type Plugin } from "vite";
import vue from "@vitejs/plugin-vue";
// @ts-expect-error type error without @types/node package
import process from "node:process";
const host = process.env.TAURI_DEV_HOST;

// Oldest webview engines the app must run in. JavaScript newer than this is
// rewritten into older, equivalent code.
// - macOS 12 webviews use the system WebKit, which is Safari 15.6 level even
//   when a newer Safari app is installed. Monaco uses `static { }` blocks in
//   classes (Safari 16.4+), which fail there without this.
// - Windows (later) uses WebView2, i.e. Chromium.
const WEBVIEW_TARGET = ["safari15", "chrome105"];

// In `tauri dev`, Vite serves some dependency files as they are (Monaco's web
// worker and the files it imports skip the pre-bundling step), so rewrite
// their syntax here too. Production builds are covered by `build.target`.
function lowerDependencySyntax(): Plugin {
  return {
    name: "moongit:lower-dependency-syntax",
    apply: "serve",
    async transform(code, id) {
      const file = id.split("?")[0];
      if (!file.includes("/node_modules/") || !/\.m?js$/.test(file)) {
        return null;
      }
      return transformWithOxc(code, file, { target: WEBVIEW_TARGET });
    },
  };
}

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [vue(), lowerDependencySyntax()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },

  // Dependencies pre-bundled for `tauri dev` must also run in the old webviews.
  optimizeDeps: {
    rolldownOptions: {
      transform: { target: WEBVIEW_TARGET },
    },
  },

  build: {
    target: WEBVIEW_TARGET,
    // Monaco (the diff editor) is one ~2.7 MB chunk, loaded only when a diff
    // is first shown. That's expected for a desktop app, so don't warn about it.
    chunkSizeWarningLimit: 3000,
  },
}));
