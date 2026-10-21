<script setup lang="ts">
import { computed } from "vue";
import type { BranchInfo } from "../types/status";

// One line under the repo name: "main → origin/main ↑2 ↓1".
const props = defineProps<{
  branch: BranchInfo;
}>();

const branchLabel = computed(() => {
  const { name, commit } = props.branch;
  if (name !== null) {
    return name;
  }
  return commit === null ? "HEAD detached" : `HEAD detached at ${commit.slice(0, 7)}`;
});

// Upstream exists and ahead/behind are meaningful.
const tracking = computed(() => props.branch.upstream !== null && !props.branch.upstreamGone);
const upToDate = computed(() => props.branch.ahead === 0 && props.branch.behind === 0);
</script>

<template>
  <p class="branch-status">
    <span class="name" :class="{ detached: branch.name === null }">⎇ {{ branchLabel }}</span>
    <span v-if="branch.commit === null" class="muted">(no commits yet)</span>

    <template v-if="branch.upstream !== null">
      <span class="muted">→</span>
      <span :class="branch.upstreamGone ? 'gone' : 'muted'">{{ branch.upstream }}</span>
      <span v-if="branch.upstreamGone" class="gone" title="This branch was deleted on the server">(gone)</span>
    </template>
    <span v-else-if="branch.name !== null" class="muted">(no upstream)</span>

    <!-- Counted against the remote-tracking branch, i.e. as of the last fetch. -->
    <span v-if="tracking" class="counts" title="Compared with the upstream as of the last fetch">
      <span v-if="upToDate" class="muted">up to date</span>
      <template v-else>
        <span v-if="branch.ahead > 0" class="ahead" :title="`${branch.ahead} commit(s) to push`">↑{{ branch.ahead }}</span>
        <span v-if="branch.behind > 0" class="behind" :title="`${branch.behind} commit(s) to pull`">↓{{ branch.behind }}</span>
      </template>
    </span>
  </p>
</template>

<style scoped>
.branch-status {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 6px;
  margin: 4px 0 0;
}

.name {
  font-weight: 600;
}

.detached {
  color: var(--warning);
}

.muted {
  color: var(--text-muted);
}

.gone {
  color: var(--danger);
}

.counts {
  display: inline-flex;
  gap: 6px;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
}

.ahead {
  color: var(--success);
}

.behind {
  color: var(--accent);
}
</style>
