<script setup lang="ts">
// Vite turns the import into the image's URL (and copies the file into the build).
import background from "../assets/background.svg";
</script>

<template>
  <!-- Shown while no repository is selected: the moon base picture, and one
       short hint below it. -->
  <section class="welcome">
    <img class="picture" :src="background" alt="" draggable="false" />
    <p class="hint">Select a repository on the left, or click + to add one.</p>
  </section>
</template>

<style scoped>
.welcome {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  height: 100%;
  padding: 16px;
  box-sizing: border-box;
}

/* The picture fades out at its edges, so it blends into the window in both
   light and dark mode. 2:1, as large as the window allows (up to 1400px). */
.picture {
  width: 100%;
  max-width: 1400px;
  max-height: 85%;
  object-fit: contain;
  user-select: none;
}

.hint {
  margin: 0;
  color: var(--text-muted);
}
</style>
