import { setTheme } from "@tauri-apps/api/app";

// View → Theme (step 33): follow macOS, or force light / dark.
//
// Tauri's app-level setTheme changes the appearance of the whole app: every
// window (also diff / conflict windows opened later) and their title bars.
// The web view inside follows it, so the CSS `prefers-color-scheme` rules in
// styles.css and the Monaco editors (monaco.ts listens to the same media
// query) switch by themselves; nothing else has to know about the choice.

export type ThemeChoice = "system" | "light" | "dark";

const KEY = "moongit.theme";

export function readThemeChoice(): ThemeChoice {
  try {
    const saved = localStorage.getItem(KEY);
    return saved === "light" || saved === "dark" ? saved : "system";
  } catch {
    return "system";
  }
}

/** Applies `choice` to the app, and remembers it for the next start. */
export async function applyThemeChoice(choice: ThemeChoice): Promise<void> {
  try {
    localStorage.setItem(KEY, choice);
  } catch {
    // Not remembered; it still applies now.
  }
  // null = follow the system.
  await setTheme(choice === "system" ? null : choice);
}
