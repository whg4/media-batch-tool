<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import type { FileInfo } from "../types";
import { formatBytes, formatDuration } from "../lib/format";
import Badge from "./ui/badge.vue";

defineProps<{ file: FileInfo }>();
const emit = defineEmits<{ remove: [id: string] }>();

const kindLabel: Record<string, { label: string; variant: "brand" | "success" | "danger" }> = {
  image: { label: "图片", variant: "brand" },
  video: { label: "视频", variant: "success" },
  unsupported: { label: "不支持", variant: "danger" },
};
</script>

<template>
  <div
    class="group flex items-center gap-3 rounded-xl border border-slate-200 bg-white p-2.5 transition-colors hover:border-brand-300 dark:border-slate-800 dark:bg-slate-900 dark:hover:border-brand-700"
  >
    <div
      class="flex h-12 w-12 shrink-0 items-center justify-center overflow-hidden rounded-lg bg-slate-100 text-xl dark:bg-slate-800"
    >
      <img
        v-if="file.thumb"
        :src="convertFileSrc(file.thumb)"
        class="h-full w-full object-cover"
        alt=""
      />
      <span v-else>{{ file.kind === "video" ? "🎬" : file.kind === "image" ? "🖼️" : "📄" }}</span>
    </div>
    <div class="min-w-0 flex-1">
      <div class="flex items-center gap-2">
        <span class="truncate text-sm font-medium" :title="file.name">{{ file.name }}</span>
        <Badge :variant="kindLabel[file.kind]?.variant ?? 'default'">
          {{ kindLabel[file.kind]?.label ?? file.format }}
        </Badge>
      </div>
      <div class="mt-0.5 text-xs text-slate-500 dark:text-slate-400">
        {{ formatBytes(file.size) }}
        <template v-if="file.width && file.height"> · {{ file.width }}×{{ file.height }}</template>
        <template v-if="file.duration_secs"> · {{ formatDuration(file.duration_secs) }}</template>
      </div>
    </div>
    <button
      class="rounded-lg p-1.5 text-slate-400 opacity-0 transition-opacity hover:bg-slate-100 hover:text-rose-500 group-hover:opacity-100 dark:hover:bg-slate-800"
      @click="emit('remove', file.id)"
      title="移除"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 6 6 18M6 6l12 12" />
      </svg>
    </button>
  </div>
</template>
