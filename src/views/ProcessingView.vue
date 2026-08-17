<script setup lang="ts">
import { computed, onMounted, watch } from "vue";
import { useRouter } from "vue-router";
import { ROUTES } from "../router";
import { useTaskStore } from "../stores/task";
import { useFilesStore } from "../stores/files";
import { t } from "../locales";
import ProgressBar from "../components/ProgressBar.vue";
import Badge from "../components/ui/badge.vue";
import Button from "../components/ui/button.vue";
import { formatBytes } from "../lib/format";

const router = useRouter();
const taskStore = useTaskStore();
const filesStore = useFilesStore();

const statusMeta = computed(() => ({
  waiting: { label: "等待中", cls: "default" as const },
  processing: { label: "处理中", cls: "brand" as const },
  done: { label: "完成", cls: "success" as const },
  skipped: { label: "已跳过", cls: "warning" as const },
  failed: { label: "失败", cls: "danger" as const },
}));

const nameOf = (id: string) =>
  filesStore.files.find((f) => f.id === id)?.name ?? taskStore.items.find((i) => i.id === id)?.name ?? "";

watch(
  () => taskStore.summary,
  (s) => {
    if (s) router.push(ROUTES.done);
  },
);

onMounted(() => {
  // batch may have already finished before this view mounted (tiny/instant jobs)
  if (taskStore.summary) router.push(ROUTES.done);
});
</script>

<template>
  <div class="flex h-full flex-col items-center justify-center gap-6 p-8">
    <div class="w-full max-w-2xl rounded-3xl border border-slate-200 bg-white p-8 shadow-sm dark:border-slate-800 dark:bg-slate-900">
      <div class="mb-6 flex items-center gap-3">
        <span class="flex h-10 w-10 animate-pulse items-center justify-center rounded-xl bg-brand-50 text-xl dark:bg-brand-950/50">⚙️</span>
        <div>
          <h2 class="text-lg font-semibold">{{ t("processing") }}</h2>
          <p class="text-sm text-slate-500">{{ t("processing_progress", { done: taskStore.done, total: taskStore.total }) }}</p>
        </div>
      </div>

      <ProgressBar :percent="taskStore.percent" class="mb-6" />

      <div class="max-h-72 space-y-1.5 overflow-y-auto pr-1">
        <div
          v-for="it in taskStore.items"
          :key="it.id"
          class="flex items-center gap-3 rounded-lg px-2 py-1.5 hover:bg-slate-50 dark:hover:bg-slate-800/50"
        >
          <span class="w-5 text-center text-sm">
            {{ it.status === "done" ? "✅" : it.status === "failed" ? "❌" : it.status === "skipped" ? "⏭️" : "⏳" }}
          </span>
          <span class="min-w-0 flex-1 truncate text-sm" :title="nameOf(it.id)">{{ nameOf(it.id) }}</span>
          <span v-if="it.status === 'processing'" class="w-12 text-right text-xs text-slate-400">{{ Math.round(it.percent) }}%</span>
          <span v-else-if="it.status === 'done' && it.saved != null" class="w-20 text-right text-xs text-emerald-600 dark:text-emerald-400">
            -{{ formatBytes(it.saved) }}
          </span>
          <Badge v-else :variant="statusMeta[it.status].cls">{{ statusMeta[it.status].label }}</Badge>
        </div>
      </div>

      <div v-if="taskStore.error" class="mt-4 rounded-lg bg-rose-50 px-3 py-2 text-sm text-rose-600 dark:bg-rose-950/50 dark:text-rose-300">
        {{ taskStore.error }}
      </div>

      <div class="mt-6 flex justify-center">
        <Button variant="danger" size="md" :disabled="!taskStore.running" @click="taskStore.cancel()">
          ■ {{ t("cancel_task") }}
        </Button>
      </div>
    </div>
  </div>
</template>
