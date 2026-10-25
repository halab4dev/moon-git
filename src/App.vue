<script setup lang="ts">
import { onMounted } from "vue";
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

const store = useRepoStore();

onMounted(() => {
  store.attempt(() => store.load());
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
