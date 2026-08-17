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

  // check for app updates (only inside the Tauri runtime)
  (async () => {
    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const update = await check();
      if (update) {
        const ok = window.confirm(`发现新版本 ${update.version}，是否下载并安装？`);
        if (ok) {
          await update.downloadAndInstall();
        }
      }
    } catch {
      // not running in Tauri, or no update endpoint configured
    }
  })();
});
</script>

<template>
  <div class="flex h-full flex-col">
    <RouterView />
  </div>
</template>
