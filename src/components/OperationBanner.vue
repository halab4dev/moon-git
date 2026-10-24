<script setup lang="ts">
// The coloured bar under the repo name while a merge or rebase waits for
// the user: blue while it only waits, red while files have conflicts.
// The text goes in the default slot, the buttons in the "actions" slot.
defineProps<{
  /** "Merging:", "Rebasing:"… */
  label: string;
  conflicts: boolean;
}>();
</script>

<template>
  <div class="banner" :class="{ conflicts }">
    <span>
      <strong>{{ label }}</strong>
      <slot></slot>
    </span>
    <span class="spacer"></span>
    <slot name="actions"></slot>
  </div>
</template>

<style scoped>
.banner {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
  padding: 6px 10px;
  border: 1px solid var(--accent);
  border-left-width: 4px;
  border-radius: 6px;
  background: var(--info-bg);
}

.banner.conflicts {
  border-color: var(--danger);
  background: var(--danger-bg);
}

strong {
  margin-right: 4px;
  color: var(--accent);
}

.banner.conflicts strong {
  color: var(--danger);
}

.spacer {
  flex: 1;
}
</style>
