import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { FileRow } from "./components/fileRows";

// Extra windows. Each one is its own webview with its own JavaScript, so it
// never blocks the main window. It loads the same index.html; the "view"
// query parameter tells main.ts which screen to show.

/** Opens (or brings to the front) a full-size diff window for one file. */
export async function openDiffWindow(repoId: string, repoName: string, row: FileRow): Promise<void> {
  const label = diffWindowLabel(repoId, row.path);

  // One window per file: double-clicking the same file again just focuses it.
  const existing = await WebviewWindow.getByLabel(label);
  if (existing !== null) {
    await existing.unminimize();
    await existing.setFocus();
    return;
  }

  const params = new URLSearchParams({ view: "diff", repoId, path: row.path });
  const window = new WebviewWindow(label, {
    url: `index.html?${params.toString()}`,
    title: `${row.name} — ${repoName}`,
    width: 1200,
    height: 850,
    minWidth: 600,
    minHeight: 400,
  });
  await new Promise<void>((resolve, reject) => {
    window.once("tauri://created", () => resolve());
    window.once("tauri://error", (event) => reject(new Error(`Could not open the diff window: ${String(event.payload)}`)));
  });
}

// Window labels may only contain letters, digits and - / : _ , so a file path
// can't be used directly. A short hash of repo + path is stable and safe.
function diffWindowLabel(repoId: string, path: string): string {
  // FNV-1a, a tiny well-known hash; collisions don't matter much here.
  let hash = 0x811c9dc5;
  for (const char of `${repoId}\n${path}`) {
    hash ^= char.codePointAt(0) ?? 0;
    hash = Math.imul(hash, 0x01000193) >>> 0;
  }
  return `diff-${hash.toString(16)}`;
}
