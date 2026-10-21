<script setup lang="ts">
// Placeholder: the repository tree arrives in step 3.
</script>

<template>
  <aside class="sidebar">
    <header class="sidebar-header">Repositories</header>
    <div class="sidebar-body">
      <p class="empty">No repositories yet.</p>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border);
}

.sidebar-header {
  padding: 10px 12px;
  font-weight: 600;
  text-transform: uppercase;
  font-size: 11px;
  letter-spacing: 0.05em;
  color: var(--text-muted);
  border-bottom: 1px solid var(--border);
}

.sidebar-body {
  flex: 1;
  overflow-y: auto;
  padding: 8px 12px;
}

.empty {
  color: var(--text-muted);
}
</style>
