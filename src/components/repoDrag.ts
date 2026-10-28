import { ref } from "vue";
import { useRepoStore } from "../stores/repos";
import type { RepoEntry } from "../types/repo";

// Dragging a repository onto a group (or out to the top level) in the
// sidebar (step 34), with the browser's own drag and drop (HTML5):
//   dragstart on the repo row → dragover on whatever is under the mouse
//   (says "a drop is possible here") → drop → move it.
//
// Where it goes:
//   - a group row            → into that group
//   - a repo row             → next to that repo (its group)
//   - empty space in the list → top level
//
// Shared by RepoItem, GroupItem and AppSidebar, hence a module with its own
// state instead of props/emit through the recursive tree.

/** The repository being dragged (null when nothing is). */
export const draggedRepo = ref<RepoEntry | null>(null);
/** Highlighted drop place: a group id, "top" for the top level, or null. */
export const dropTarget = ref<string | null>(null);

// Our own type in the drag data: other drags (text, files) are ignored.
const DRAG_TYPE = "application/x-moongit-repo";
const TOP = "top";

export function startRepoDrag(event: DragEvent, repo: RepoEntry) {
  if (event.dataTransfer === null) {
    return;
  }
  event.dataTransfer.setData(DRAG_TYPE, repo.id);
  event.dataTransfer.effectAllowed = "move";
  draggedRepo.value = repo;
}

export function endRepoDrag() {
  draggedRepo.value = null;
  dropTarget.value = null;
  clearExpandTimer();
}

/**
 * dragover on a place that would put the repo in `groupId` (null = top level).
 * Accepting means calling preventDefault(); otherwise the browser shows "no drop".
 */
export function overDropPlace(event: DragEvent, groupId: string | null) {
  const repo = draggedRepo.value;
  if (repo === null) {
    return;
  }
  // Handled here: the list behind this row must not take over the target.
  event.stopPropagation();
  if (repo.groupId === groupId) {
    // Already there: nothing to do, show no target.
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
  const repo = draggedRepo.value;
  event.stopPropagation();
  endRepoDrag();
  if (repo === null || repo.groupId === groupId) {
    return;
  }
  event.preventDefault();
  const store = useRepoStore();
  // moveRepo also opens the group (and its parents), so the repo stays visible.
  await store.attempt(() => store.moveRepo(repo.id, groupId));
}

export function isDropTarget(groupId: string | null): boolean {
  return draggedRepo.value !== null && dropTarget.value === (groupId ?? TOP);
}

// ---- hovering over a closed group opens it after a moment ----

let expandTimer: ReturnType<typeof setTimeout> | null = null;

export function hoverClosedGroup(groupId: string) {
  clearExpandTimer();
  expandTimer = setTimeout(() => {
    if (draggedRepo.value !== null) {
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
