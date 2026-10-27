import { watchEffect } from "vue";
import { getVersion } from "@tauri-apps/api/app";
import { CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu } from "@tauri-apps/api/menu";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { addLocalRepo, newGroup } from "./components/sidebarMenus";
import { useDialogStore } from "./stores/dialogs";
import { useMenuCommands, type CommandId } from "./stores/menuCommands";
import { useRepoStore } from "./stores/repos";
import { applyThemeChoice, readThemeChoice, type ThemeChoice } from "./theme";

// The menu bar of the app (step 31a), built from JavaScript and set as the
// app menu. Our own items only send a command to the menuCommands store;
// the component that registered it (RepoView, SyncButtons, BranchPanel…)
// does the work, exactly as its button would. Items are greyed out while
// nobody can run their command.
//
// The standard macOS items (Edit's Copy/Paste, Hide, Quit, Minimize…) are
// "predefined": macOS itself handles them. Edit must stay: without it,
// ⌘C / ⌘V don't work in text fields on macOS.

const APP_NAME = "MoonGit";

/** Builds the menu bar and keeps its items enabled/disabled. Returns a function that stops the updates. */
export async function installAppMenu(): Promise<() => void> {
  const commands = useMenuCommands();
  registerAppCommands();

  const items = new Map<CommandId, MenuItem>();
  const item = async (command: CommandId, text: string, accelerator?: string) => {
    const created = await MenuItem.new({
      id: command,
      text,
      accelerator,
      enabled: false,
      action: () => commands.run(command),
    });
    items.set(command, created);
    return created;
  };
  const separator = () => PredefinedMenuItem.new({ item: "Separator" });
  const predefined = (kind: "Undo" | "Redo" | "Cut" | "Copy" | "Paste" | "SelectAll") =>
    PredefinedMenuItem.new({ item: kind });

  // On macOS the first submenu is always the app menu. The default labels
  // use the program's file name ("moongit" → "Quit Moongit"), so the
  // app's real name is written out here.
  const appSubmenu = await Submenu.new({
    text: APP_NAME,
    items: [
      await PredefinedMenuItem.new({
        // The version comes from tauri.conf.json ("0.1.0").
        item: { About: { name: APP_NAME, version: await getVersion() } },
        text: `About ${APP_NAME}`,
      }),
      await separator(),
      await PredefinedMenuItem.new({ item: "Services" }),
      await separator(),
      await PredefinedMenuItem.new({ item: "Hide", text: `Hide ${APP_NAME}` }),
      await PredefinedMenuItem.new({ item: "HideOthers" }),
      await PredefinedMenuItem.new({ item: "ShowAll" }),
      await separator(),
      await PredefinedMenuItem.new({ item: "Quit", text: `Quit ${APP_NAME}` }),
    ],
  });

  const repository = await Submenu.new({
    text: "Repository",
    items: [
      await item("repo.addLocal", "Add Local Repository…", "CmdOrCtrl+O"),
      await item("repo.clone", "Clone Repository…", "CmdOrCtrl+Shift+O"),
      await item("repo.newGroup", "New Group…"),
      await separator(),
      await item("repo.remove", "Remove from List"),
      await separator(),
      await PredefinedMenuItem.new({ item: "CloseWindow" }),
    ],
  });

  const edit = await Submenu.new({
    text: "Edit",
    items: [
      await predefined("Undo"),
      await predefined("Redo"),
      await separator(),
      await predefined("Cut"),
      await predefined("Copy"),
      await predefined("Paste"),
      await predefined("SelectAll"),
      await separator(),
      await item("edit.copyPath", "Copy Repository Path", "CmdOrCtrl+Alt+C"),
    ],
  });

  // Theme (step 33): three check items that behave like radio buttons.
  // The saved choice is applied at start (App.vue), so only the marks are set here.
  const themeItems = new Map<ThemeChoice, CheckMenuItem>();
  const chooseTheme = async (choice: ThemeChoice) => {
    // A check item flips itself when clicked: set all marks explicitly.
    for (const [other, menuItem] of themeItems) {
      await menuItem.setChecked(other === choice);
    }
    await applyThemeChoice(choice);
  };
  const saved = readThemeChoice();
  for (const [choice, text] of [
    ["system", "System"],
    ["light", "Light"],
    ["dark", "Dark"],
  ] as const) {
    themeItems.set(
      choice,
      await CheckMenuItem.new({ text, checked: choice === saved, action: () => void chooseTheme(choice) }),
    );
  }
  const theme = await Submenu.new({ text: "Theme", items: [...themeItems.values()] });

  const view = await Submenu.new({
    text: "View",
    items: [
      await item("view.changes", "Changes", "CmdOrCtrl+1"),
      await item("view.history", "History", "CmdOrCtrl+2"),
      await item("view.stashes", "Stashes", "CmdOrCtrl+3"),
      await separator(),
      await item("view.refresh", "Refresh", "CmdOrCtrl+R"),
      await separator(),
      theme,
      await separator(),
      await PredefinedMenuItem.new({ item: "Fullscreen" }),
      // ⌘R is Refresh now; during development the page can still be reloaded.
      ...(import.meta.env.DEV
        ? [
            await separator(),
            await MenuItem.new({
              text: "Reload Interface (dev)",
              accelerator: "CmdOrCtrl+Shift+R",
              action: () => window.location.reload(),
            }),
          ]
        : []),
    ],
  });

  const remote = await Submenu.new({
    text: "Remote",
    items: [
      await item("remote.fetch", "Fetch", "CmdOrCtrl+Shift+F"),
      await item("remote.pull", "Pull", "CmdOrCtrl+Shift+L"),
      await item("remote.push", "Push", "CmdOrCtrl+Shift+P"),
      await item("remote.forcePush", "Force Push…"),
      await separator(),
      await item("remote.addRemote", "Add Remote…"),
    ],
  });

  const local = await Submenu.new({
    text: "Local",
    items: [
      await item("local.commit", "Commit…", "CmdOrCtrl+K"),
      await separator(),
      await item("local.stage", "Stage"),
      await item("local.unstage", "Unstage"),
      await item("local.discard", "Discard…"),
      await separator(),
      await item("local.stash", "Stash…", "CmdOrCtrl+Shift+S"),
      await item("local.apply", "Apply Stash"),
      await item("local.applyDrop", "Apply and Drop Stash"),
      await item("local.drop", "Drop Stash…"),
    ],
  });

  const branch = await Submenu.new({
    text: "Branch",
    items: [
      await item("branch.checkout", "Checkout…", "CmdOrCtrl+Shift+C"),
      await item("branch.new", "New Branch…", "CmdOrCtrl+Shift+N"),
      await item("branch.rename", "Rename Current Branch…"),
    ],
  });

  const windowMenu = await Submenu.new({
    text: "Window",
    items: [
      await PredefinedMenuItem.new({ item: "Minimize" }),
      await PredefinedMenuItem.new({ item: "Maximize", text: "Zoom" }),
      await separator(),
      await PredefinedMenuItem.new({ item: "BringAllToFront" }),
    ],
  });

  const menu = await Menu.new({ items: [appSubmenu, repository, edit, view, remote, local, branch, windowMenu] });
  await menu.setAsAppMenu();
  // macOS adds the list of open windows (main, diff, conflict…) to this menu.
  await windowMenu.setAsWindowsMenuForNSApp().catch(() => {});

  // Follow the commands: an item is enabled while its command can run.
  const shown = new Map<CommandId, boolean>();
  return watchEffect(() => {
    for (const [command, menuItem] of items) {
      const enabled = commands.isEnabled(command);
      if (shown.get(command) !== enabled) {
        shown.set(command, enabled);
        void menuItem.setEnabled(enabled);
      }
    }
  });
}

/** Commands that don't belong to one component: the repository list and the clipboard. */
function registerAppCommands() {
  const commands = useMenuCommands();
  const store = useRepoStore();
  const selectedGroup = () => store.selected?.groupId ?? null;

  commands.register("repo.addLocal", { run: () => addLocalRepo(selectedGroup()) });
  commands.register("repo.clone", { run: () => useDialogStore().openClone(selectedGroup()) });
  commands.register("repo.newGroup", { run: () => newGroup(null) });
  commands.register("repo.remove", {
    // Only forgets the repo; the folder stays on disk (like the sidebar's menu).
    run: () => {
      const repo = store.selected;
      return repo === null ? undefined : store.attempt(() => store.removeRepo(repo.id));
    },
    enabled: () => store.selected !== null,
  });
  commands.register("edit.copyPath", {
    run: () => {
      const repo = store.selected;
      return repo === null ? undefined : writeText(repo.path);
    },
    enabled: () => store.selected !== null,
  });
}
