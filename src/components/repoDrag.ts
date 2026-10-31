import { ref } from "vue";
import { useRepoStore } from "../stores/repos";
import type { RepoEntry, RepoGroup } from "../types/repo";

// Drag and drop in the sidebar, with the browser's own drag and drop (HTML5):
// repositories (step 34) and whole groups (step 35a).
//   dragstart on a row → dragover on whatever is under the mouse (says "a
//   drop is possible here") → drop → move it.
//
// Where it goes:
//   - a group row             → into that group
//   - a repository row        → next to that repository (its group)
//   - empty space in the list → top level
//
// A group can't go into itself or one of its own subgroups.
// Shared by RepoItem, GroupItem and AppSidebar, hence a module with its own
// state instead of props/emit through the recursive tree.

/** What is being dragged (null when nothing is). */
export type Dragged = { kind: "repos"; repos: RepoEntry[] } | { kind: "group"; group: RepoGroup };

export const dragged = ref<Dragged | null>(null);
/** Highlighted drop place: a group id, "top" for the top level, or null. */
export const dropTarget = ref<string | null>(null);

// Our own type in the drag data: other drags (text, files) are ignored.
const DRAG_TYPE = "application/x-moongit-item";
const TOP = "top";

function begin(event: DragEvent, item: Dragged, id: string) {
  if (event.dataTransfer === null) {
    return;
  }
  event.dataTransfer.setData(DRAG_TYPE, id);
  event.dataTransfer.effectAllowed = "move";
  dragged.value = item;
}

/** Dragging a picked repository takes all the picked ones along (step 35b). */
export function startRepoDrag(event: DragEvent, repo: RepoEntry) {
  const store = useRepoStore();
  const repos = store.isPicked(repo.id) && store.pickedRepos.length > 1 ? store.pickedRepos : [repo];
  begin(event, { kind: "repos", repos }, repo.id);
}

export function startGroupDrag(event: DragEvent, group: RepoGroup) {
  // Only the group row itself, not the rows inside it.
  event.stopPropagation();
  begin(event, { kind: "group", group }, group.id);
}

export function endDrag() {
  dragged.value = null;
  dropTarget.value = null;
  clearExpandTimer();
}

/** Would dropping here (into `groupId`, null = top level) change anything, and is it allowed? */
function canDropInto(groupId: string | null): boolean {
  const item = dragged.value;
  if (item === null) {
    return false;
  }
  if (item.kind === "repos") {
    return item.repos.some((repo) => repo.groupId !== groupId);
  }
  if (item.group.parentId === groupId) {
    return false; // already there
  }
  return groupId === null || !isInside(groupId, item.group.id);
}

/** Is group `id` the group `ancestor` itself, or somewhere below it? */
function isInside(id: string, ancestor: string): boolean {
  const groups = useRepoStore().groups;
  let current: string | null = id;
  for (let step = 0; current !== null && step <= groups.length; step++) {
    if (current === ancestor) {
      return true;
    }
    const found: RepoGroup | undefined = groups.find((g) => g.id === current);
    current = found?.parentId ?? null;
  }
  return false;
}

/**
 * dragover on a place that would put the item in `groupId` (null = top level).
 * Accepting means calling preventDefault(); otherwise the browser shows "no drop".
 */
export function overDropPlace(event: DragEvent, groupId: string | null) {
  if (dragged.value === null) {
    return;
  }
  // Handled here: the list behind this row must not take over the target.
  event.stopPropagation();
  if (!canDropInto(groupId)) {
    dropTarget.value = null;
    return;
  }
  event.preventDefault();
  if (event.dataTransfer !== null) {
    event.dataTransfer.dropEffect = "move";
  }
  dropTarget.value = groupId ?? TOP;
}

export async function dropOnPlace(event: DragEvent, groupId: string | null) {
  const item = dragged.value;
  const allowed = canDropInto(groupId);
  event.stopPropagation();
  endDrag();
  if (item === null || !allowed) {
    return;
  }
  event.preventDefault();
  const store = useRepoStore();
  // moveRepo / moveGroup also open the target group, so the moved item stays visible.
  if (item.kind === "group") {
    await store.attempt(() => store.moveGroup(item.group.id, groupId));
    return;
  }
  await store.attempt(async () => {
    for (const repo of item.repos.filter((r) => r.groupId !== groupId)) {
      await store.moveRepo(repo.id, groupId);
    }
  });
}

export function isDropTarget(groupId: string | null): boolean {
  return dragged.value !== null && dropTarget.value === (groupId ?? TOP);
}

/** Is this row being dragged (shown faded)? */
export function isDragged(kind: "repo" | "group", id: string): boolean {
  const item = dragged.value;
  if (item === null) {
    return false;
  }
  return kind === "group"
    ? item.kind === "group" && item.group.id === id
    : item.kind === "repos" && item.repos.some((repo) => repo.id === id);
}

// ---- hovering over a closed group opens it after a moment ----

let expandTimer: ReturnType<typeof setTimeout> | null = null;

export function hoverClosedGroup(groupId: string) {
  clearExpandTimer();
  expandTimer = setTimeout(() => {
    // Not while dragging that group itself: it would open under the mouse.
    if (dragged.value !== null && canDropInto(groupId)) {
      useRepoStore().revealGroup(groupId);
    }
  }, 700);
}

export function clearExpandTimer() {
  if (expandTimer !== null) {
    clearTimeout(expandTimer);
    expandTimer = null;
  }
}
