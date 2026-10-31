import { computed, type ComputedRef } from "vue";
import { useRepoStore } from "../stores/repos";
import { countReposInGroup } from "../stores/repoTree";
import type { RepoGroup } from "../types/repo";

// What a "write into several repositories" dialog works on (steps 14b / 15b
// / 35c): a group (with its subgroups), or the repositories picked in the
// sidebar. Shared by the Commit Identity and the Login dialogs.

/** The dialog request: one of the two. */
export type ApplyTargetRequest = { groupId: string } | { repoIds: string[] };

export function useApplyTarget(request: () => ApplyTargetRequest | null) {
  const store = useRepoStore();

  const group: ComputedRef<RepoGroup | null> = computed(() => {
    const current = request();
    if (current === null || !("groupId" in current)) {
      return null;
    }
    return store.groups.find((g) => g.id === current.groupId) ?? null;
  });

  const repoIds = computed(() => {
    const current = request();
    return current !== null && "repoIds" in current ? current.repoIds : null;
  });

  /** For titles: `group "Work"` or `3 repositories`. */
  const label = computed(() =>
    group.value !== null ? `group "${group.value.name}"` : `${repoIds.value?.length ?? 0} repositories`,
  );

  const repoCount = computed(() =>
    group.value !== null
      ? countReposInGroup(store.groups, store.repos, group.value.id)
      : (repoIds.value?.length ?? 0),
  );

  /** "of this group and its subgroups" / "selected", after "the N repositories". */
  const where = computed(() => (group.value !== null ? "of this group and its subgroups" : "selected in the sidebar"));

  /** Open, and what it points at still exists. */
  const ready = computed(() => request() !== null && (group.value !== null || repoIds.value !== null));

  return { group, repoIds, label, repoCount, where, ready };
}
