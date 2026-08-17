<script setup lang="ts">
defineProps<{ percent: number; class?: string }>();
</script>

<template>
  <div class="h-2 w-full overflow-hidden rounded-full bg-slate-200 dark:bg-slate-800" :class="$props.class">
    <div
      class="h-full rounded-full bg-gradient-to-r from-brand-500 to-brand-400 transition-[width] duration-300 ease-out"
      :style="{ width: `${Math.min(100, Math.max(0, percent))}%` }"
    />
  </div>
</template>
