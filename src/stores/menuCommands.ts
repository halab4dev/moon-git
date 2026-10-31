import { acceptHMRUpdate, defineStore } from "pinia";
import { getCurrentInstance, onUnmounted, shallowReactive } from "vue";

// The menu bar (appMenu.ts) and the components that can carry out its
// commands meet here. A component registers what it can do ("fetch",
// "commit"…) while it is on screen; a menu item is enabled only while
// someone has registered its command and says it can run now. So the menu
// never needs to know which component does what.

export type CommandId =
  | "repo.addLocal"
  | "repo.clone"
  | "repo.newGroup"
  | "repo.remove"
  | "repo.identity"
  | "edit.copyPath"
  | "view.changes"
  | "view.history"
  | "view.stashes"
  | "view.refresh"
  | "remote.fetch"
  | "remote.pull"
  | "remote.push"
  | "remote.forcePush"
  | "remote.addRemote"
  | "local.commit"
  | "local.stage"
  | "local.unstage"
  | "local.discard"
  | "local.stash"
  | "local.apply"
  | "local.applyDrop"
  | "local.drop"
  | "branch.checkout"
  | "branch.new"
  | "branch.rename";

export interface CommandHandler {
  run: () => void | Promise<unknown>;
  /** Can it run right now? Reactive: the menu item follows it. Default: always. */
  enabled?: () => boolean;
}

export const useMenuCommands = defineStore("menuCommands", () => {
  // shallowReactive: Vue notices handlers being added or removed (that is
  // all we need; the handlers themselves are plain objects).
  const handlers = shallowReactive(new Map<CommandId, CommandHandler>());

  /** Returns the function that removes it again. */
  function register(id: CommandId, handler: CommandHandler): () => void {
    handlers.set(id, handler);
    return () => {
      // Only if it is still ours (a newer component may have taken over).
      if (handlers.get(id) === handler) {
        handlers.delete(id);
      }
    };
  }

  function isEnabled(id: CommandId): boolean {
    const handler = handlers.get(id);
    return handler !== undefined && (handler.enabled?.() ?? true);
  }

  function run(id: CommandId) {
    // Checked again: a keyboard shortcut can fire before the menu item is greyed out.
    if (isEnabled(id)) {
      void handlers.get(id)?.run();
    }
  }

  return { register, isEnabled, run };
});

/**
 * For a component: registers its commands while it is mounted. Call it in
 * `<script setup>`.
 */
export function useMenuHandlers(handlers: Partial<Record<CommandId, CommandHandler>>) {
  const commands = useMenuCommands();
  const removers = Object.entries(handlers).map(([id, handler]) =>
    commands.register(id as CommandId, handler as CommandHandler),
  );
  if (getCurrentInstance() !== null) {
    onUnmounted(() => removers.forEach((remove) => remove()));
  }
}

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useMenuCommands, import.meta.hot));
}
