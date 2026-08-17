<script setup lang="ts">
import { onMounted } from "vue";
import { RouterView } from "vue-router";

onMounted(() => {
  // follow system color scheme
  const mq = window.matchMedia("(prefers-color-scheme: dark)");
  const apply = () => {
    document.documentElement.classList.toggle("dark", mq.matches);
  };
  apply();
  mq.addEventListener("change", apply);
});
</script>

<template>
  <div class="flex h-full flex-col">
    <RouterView />
  </div>
</template>
