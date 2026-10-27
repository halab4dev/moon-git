<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import AppSidebar from "./components/AppSidebar.vue";
import CloneDialog from "./components/CloneDialog.vue";
import MergeDialog from "./components/MergeDialog.vue";
import ResetDialog from "./components/ResetDialog.vue";
import CherryPickDialog from "./components/CherryPickDialog.vue";
import NameDialog from "./components/NameDialog.vue";
import AddRemoteDialog from "./components/AddRemoteDialog.vue";
import RepoView from "./views/RepoView.vue";
import WelcomeView from "./views/WelcomeView.vue";
import { useRepoStore } from "./stores/repos";
import { installAppMenu } from "./appMenu";
import { applyThemeChoice, readThemeChoice } from "./theme";

const store = useRepoStore();

// The menu bar (step 31a) belongs to the main window; it stops following
// the app's state when this window goes away (or is hot-reloaded).
let stopMenu: (() => void) | null = null;
let unmounted = false;

onMounted(async () => {
  store.attempt(() => store.load());
  // The theme chosen last time (View → Theme); "system" = follow macOS.
  void applyThemeChoice(readThemeChoice());
  const stop = await installAppMenu();
  if (unmounted) {
    stop();
  } else {
    stopMenu = stop;
  }
});

onUnmounted(() => {
  unmounted = true;
  stopMenu?.();
});
</script>

<template>
  <div class="layout">
    <AppSidebar />
    <main class="content">
      <!-- :key makes Vue build a fresh RepoView when switching repos. -->
      <RepoView v-if="store.selected" :key="store.selected.id" :repo="store.selected" />
      <WelcomeView v-else />
    </main>
  </div>
  <NameDialog />
  <CloneDialog />
  <MergeDialog />
  <ResetDialog />
  <CherryPickDialog />
  <AddRemoteDialog />
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
