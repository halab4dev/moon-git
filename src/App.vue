<script setup lang="ts">
import AppSidebar from "./components/AppSidebar.vue";
import WelcomeView from "./views/WelcomeView.vue";
</script>

<template>
  <div class="layout">
    <AppSidebar />
    <main class="content">
      <WelcomeView />
    </main>
  </div>
</template>

<style scoped>
.layout {
  display: grid;
  grid-template-columns: 260px 1fr;
  height: 100%;
}

.content {
  overflow: auto;
}
</style>
