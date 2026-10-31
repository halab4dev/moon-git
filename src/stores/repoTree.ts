import type { RepoEntry, RepoGroup } from "../types/repo";

// Rust stores groups and repos as flat lists with parent ids.
// These helpers turn them into the nested shape the sidebar draws.

export interface GroupNode {
  group: RepoGroup;
  children: GroupNode[];
  repos: RepoEntry[];
}

export interface RepoTree {
  groups: GroupNode[];
  repos: RepoEntry[];
}

/** A group as a menu choice, e.g. { label: "Work / Client A" }. */
export interface GroupOption {
  id: string;
  label: string;
}

function byName(a: { name: string }, b: { name: string }): number {
  return a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
}

export function buildTree(groups: RepoGroup[], repos: RepoEntry[]): RepoTree {
  const nodes = new Map<string, GroupNode>();
  for (const group of groups) {
    nodes.set(group.id, { group, children: [], repos: [] });
  }

  // Anything whose parent is missing goes to the top level instead of disappearing.
  const tree: RepoTree = { groups: [], repos: [] };
  for (const node of nodes.values()) {
    const parent = node.group.parentId ? nodes.get(node.group.parentId) : undefined;
    (parent ? parent.children : tree.groups).push(node);
  }
  for (const repo of repos) {
    const parent = repo.groupId ? nodes.get(repo.groupId) : undefined;
    (parent ? parent.repos : tree.repos).push(repo);
  }

  sortLevel(tree.groups, tree.repos);
  return tree;
}

function sortLevel(groups: GroupNode[], repos: RepoEntry[]) {
  groups.sort((a, b) => byName(a.group, b.group));
  repos.sort(byName);
  for (const node of groups) {
    sortLevel(node.children, node.repos);
  }
}

/** Every group in tree order, labelled with its full path. */
export function listGroupOptions(tree: RepoTree): GroupOption[] {
  const result: GroupOption[] = [];
  const visit = (nodes: GroupNode[], prefix: string) => {
    for (const node of nodes) {
      const label = prefix + node.group.name;
      result.push({ id: node.group.id, label });
      visit(node.children, `${label} / `);
    }
  };
  visit(tree.groups, "");
  return result;
}

/** How many repositories a group holds, its subgroups included (steps 14b / 15b). */
export function countReposInGroup(groups: RepoGroup[], repos: RepoEntry[], groupId: string): number {
  const ids = new Set([groupId]);
  let added = true;
  while (added) {
    added = false;
    for (const g of groups) {
      if (g.parentId !== null && ids.has(g.parentId) && !ids.has(g.id)) {
        ids.add(g.id);
        added = true;
      }
    }
  }
  return repos.filter((r) => r.groupId !== null && ids.has(r.groupId)).length;
}

/**
 * Repository ids in the order the sidebar shows them, skipping the inside of
 * collapsed groups (step 35b, for ⇧-click ranges). At each level: groups
 * first, then repositories, like the sidebar.
 */
export function visibleRepoOrder(tree: RepoTree, isCollapsed: (groupId: string) => boolean): string[] {
  const order: string[] = [];
  const visit = (groups: GroupNode[], repos: RepoEntry[]) => {
    for (const node of groups) {
      if (!isCollapsed(node.group.id)) {
        visit(node.children, node.repos);
      }
    }
    order.push(...repos.map((repo) => repo.id));
  };
  visit(tree.groups, tree.repos);
  return order;
}

/** The ids from `from` to `to` (either order), both included; just `to` if `from` isn't visible. */
export function rangeBetween(order: string[], from: string | null, to: string): string[] {
  const end = order.indexOf(to);
  const start = from === null ? -1 : order.indexOf(from);
  if (end < 0) {
    return [];
  }
  if (start < 0) {
    return [to];
  }
  return order.slice(Math.min(start, end), Math.max(start, end) + 1);
}
