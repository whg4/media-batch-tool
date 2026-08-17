<script setup lang="ts">
import { useRouter } from "vue-router";
import { ROUTES } from "../router";
import { useFilesStore } from "../stores/files";
import { useModeStore } from "../stores/mode";
import { useFileDrop } from "../composables/useFileDrop";
import { api } from "../lib/ipc";
import { t } from "../locales";
import FileCard from "../components/FileCard.vue";
import Button from "../components/ui/button.vue";
import { cn } from "../lib/cn";

const router = useRouter();
const filesStore = useFilesStore();
const modeStore = useModeStore();
const { dragging, onDragEnter, onDragOver, onDragLeave, onDrop } = useFileDrop();

async function pickFiles() {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const sel = await open({
    multiple: true,
    filters: [
      {
        name: "媒体文件",
        extensions: [
          "jpg", "jpeg", "png", "webp", "gif", "bmp", "tiff", "tif",
          "heic", "heif", "avif", "ico",
          "mp4", "mov", "mkv", "avi", "webm", "flv", "wmv", "m4v", "3gp", "ts", "m2ts", "mts",
        ],
      },
    ],
  });
  if (sel && sel.length > 0) await filesStore.addPaths(sel);
}

async function pickFolder() {
  const picked = await api.pickFolder();
  if (picked) await filesStore.addPaths([picked]);
}

function next() {
  if (filesStore.supportedCount > 0) router.push(ROUTES.template);
}
</script>

<template>
  <div class="flex h-full flex-col p-6">
    <div class="mb-4 flex items-center justify-between">
      <button
        class="flex items-center gap-1 text-sm text-slate-500 hover:text-slate-800 dark:hover:text-slate-200"
        @click="router.push(ROUTES.home)"
      >
        ← {{ t("back") }}
      </button>
      <span class="text-sm font-medium text-slate-400">
        {{ modeStore.mode === "slim" ? t("mode_slim") : modeStore.mode === "convert" ? t("mode_convert") : t("mode_social") }}
      </span>
      <button
        v-if="filesStore.files.length > 0"
        class="text-sm text-rose-500 hover:text-rose-600"
        @click="filesStore.clear()"
      >
        {{ t("clear_all") }}
      </button>
    </div>

    <div
      class="flex min-h-64 flex-1 cursor-pointer flex-col items-center justify-center gap-3 rounded-3xl border-2 border-dashed p-8 text-center transition-all"
      :class="
        cn(
          dragging
            ? 'scale-[1.01] border-brand-500 bg-brand-50 dark:bg-brand-950/30'
            : 'border-slate-300 bg-white dark:border-slate-700 dark:bg-slate-900',
        )
      "
      @dragenter="onDragEnter"
      @dragover="onDragOver"
      @dragleave="onDragLeave"
      @drop="onDrop"
    >
      <span class="text-5xl">{{ dragging ? "📥" : "📁" }}</span>
      <p class="text-lg font-semibold">{{ t("drop_title") }}</p>
      <p class="text-sm text-slate-500 dark:text-slate-400">{{ t("drop_sub") }}</p>
      <p class="text-xs text-slate-400">{{ t("drop_hint") }}</p>
      <div class="mt-2 flex gap-3">
        <Button variant="secondary" size="sm" @click="pickFiles">{{ t("choose_files") }}</Button>
        <Button variant="secondary" size="sm" @click="pickFolder">{{ t("choose_folder") }}</Button>
      </div>
    </div>

    <div
      v-if="filesStore.files.length > 0"
      class="mt-4 flex-1 space-y-2 overflow-y-auto pr-1"
      style="max-height: 38vh"
    >
      <FileCard
        v-for="f in filesStore.files"
        :key="f.id"
        :file="f"
        @remove="filesStore.remove"
      />
    </div>

    <div class="mt-4 flex items-center justify-between">
      <span class="text-sm text-slate-500">
        {{ filesStore.files.length }} 个文件 · {{ filesStore.supportedCount }} 个可处理
        <template v-if="filesStore.unsupportedCount > 0"> · {{ filesStore.unsupportedCount }} 个{{ t("unsupported") }}</template>
      </span>
      <Button :disabled="filesStore.supportedCount === 0" size="lg" @click="next">
        {{ t("next_step") }} →
      </Button>
    </div>
  </div>
</template>
