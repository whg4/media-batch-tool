<script setup lang="ts">
import type { TemplateKind } from "../types";

defineProps<{ icon: string; title: string; desc: string; kind: TemplateKind }>();
const emit = defineEmits<{ select: [kind: TemplateKind] }>();
</script>

<template>
  <button
    class="group flex flex-col items-start gap-2 rounded-3xl border border-slate-200 bg-white p-6 text-left shadow-sm transition-all hover:-translate-y-0.5 hover:border-brand-400 hover:shadow-lg hover:shadow-brand-500/10 dark:border-slate-800 dark:bg-slate-900 dark:hover:border-brand-600"
    @click="emit('select', kind)"
  >
    <span class="flex h-12 w-12 items-center justify-center rounded-2xl bg-brand-50 text-2xl transition-transform group-hover:scale-110 dark:bg-brand-950/50">
      {{ icon }}
    </span>
    <span class="mt-1 text-lg font-semibold">{{ title }}</span>
    <span class="text-sm leading-relaxed text-slate-500 dark:text-slate-400">{{ desc }}</span>
    <span class="mt-2 text-sm font-medium text-brand-600 opacity-0 transition-opacity group-hover:opacity-100 dark:text-brand-400">
      开始 →
    </span>
  </button>
</template>
