<script setup lang="ts">
import type { Template } from "../types";
import { cn } from "../lib/cn";

defineProps<{ template: Template; selected?: boolean; deletable?: boolean }>();
const emit = defineEmits<{ select: []; remove: []; edit: [] }>();
</script>

<template>
  <button
    class="group relative flex flex-col items-start gap-1.5 rounded-2xl border p-4 text-left transition-all"
    :class="
      cn(
        selected
          ? 'border-brand-500 bg-brand-50 shadow-sm shadow-brand-500/10 dark:border-brand-500 dark:bg-brand-950/40'
          : 'border-slate-200 bg-white hover:border-brand-300 hover:shadow-sm dark:border-slate-800 dark:bg-slate-900 dark:hover:border-brand-700',
      )
    "
    @click="emit('select')"
  >
    <span v-if="selected" class="absolute right-3 top-3 flex h-5 w-5 items-center justify-center rounded-full bg-brand-600 text-white">
      <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
        <path d="M20 6 9 17l-5-5" />
      </svg>
    </span>
    <span class="text-2xl">{{ template.icon }}</span>
    <span class="font-semibold">{{ template.name }}</span>
    <span class="text-xs leading-relaxed text-slate-500 dark:text-slate-400">{{ template.description }}</span>
    <div v-if="deletable" class="absolute bottom-3 right-3 flex gap-1 opacity-0 transition-opacity group-hover:opacity-100">
      <button class="rounded-lg px-1.5 py-0.5 text-xs text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800" @click.stop="emit('edit')">编辑</button>
      <button class="rounded-lg px-1.5 py-0.5 text-xs text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-950" @click.stop="emit('remove')">删除</button>
    </div>
  </button>
</template>
