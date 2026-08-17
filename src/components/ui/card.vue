<script setup lang="ts">
import { cn } from "../../lib/cn";
defineProps<{ class?: string }>();
</script>

<template>
  <div
    :class="
      cn(
        'rounded-2xl border border-slate-200 bg-white shadow-sm dark:border-slate-800 dark:bg-slate-900',
        $props.class,
      )
    "
  >
    <slot />
  </div>
</template>
