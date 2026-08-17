<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { ROUTES } from "../router";
import { useTaskStore } from "../stores/task";
import { useFilesStore } from "../stores/files";
import { api } from "../lib/ipc";
import { t } from "../locales";
import { formatBytes } from "../lib/format";
import Button from "../components/ui/button.vue";
import Badge from "../components/ui/badge.vue";

const router = useRouter();
const taskStore = useTaskStore();
const filesStore = useFilesStore();
const replaceOriginal = ref(false);
const exportedMsg = ref("");

const summary = computed(() => taskStore.summary);
const doneItems = computed(() =>
  (summary.value?.items ?? []).filter((i) => i.output_path != null && !i.error),
);

async function exportFiles() {
  const target = await api.pickFolder();
  if (!target) return;
  const ids = doneItems.value.map((i) => i.id);
  if (ids.length === 0) return;
  const res = await api.exportFiles(ids, target, replaceOriginal.value);
  exportedMsg.value = t("exported", { n: res.exported });
  if (res.errors.length > 0) {
    exportedMsg.value += " · " + res.errors.map(([, e]) => e).join("; ");
  }
}

async function retryFailed() {
  const failedIds = (summary.value?.items ?? [])
    .filter((i) => i.error && !i.skipped)
    .map((i) => i.id);
  if (failedIds.length === 0) return;
  // put failed files back into the files store context
  const failedFiles = failedIds
    .map((id) => filesStore.files.find((f) => f.id === id))
    .filter(Boolean) as typeof filesStore.files;
  await taskStore.start(failedFiles.map((f) => f.id), taskStore.lastTemplateId ?? "");
  router.push(ROUTES.processing);
}

function goHome() {
  taskStore.reset();
  filesStore.clear();
  router.push(ROUTES.home);
}

</script>

<template>
  <div class="flex h-full flex-col items-center justify-center gap-5 overflow-y-auto p-8">
    <div class="w-full max-w-3xl rounded-3xl border border-slate-200 bg-white p-8 shadow-sm dark:border-slate-800 dark:bg-slate-900">
      <div class="mb-2 flex items-center gap-3">
        <span class="flex h-12 w-12 items-center justify-center rounded-2xl bg-emerald-50 text-2xl dark:bg-emerald-950">🎉</span>
        <div>
          <h2 class="text-xl font-bold">{{ t("done_title") }}</h2>
          <p class="text-sm text-slate-500">
            {{ t("saved_total", { size: formatBytes(summary?.saved_bytes ?? 0) }) }}
          </p>
        </div>
      </div>

      <div class="mb-4 flex gap-3 text-sm">
        <Badge variant="success">{{ t("processed") }}：{{ summary?.succeeded ?? 0 }}</Badge>
        <Badge v-if="(summary?.skipped ?? 0) > 0" variant="warning">{{ t("skipped") }}：{{ summary?.skipped }}</Badge>
        <Badge v-if="(summary?.failed ?? 0) > 0" variant="danger">{{ t("failed") }}：{{ summary?.failed }}</Badge>
      </div>

      <div class="max-h-64 space-y-1.5 overflow-y-auto pr-1">
        <div
          v-for="it in summary?.items ?? []"
          :key="it.id"
          class="flex items-center gap-3 rounded-lg px-2 py-1.5 hover:bg-slate-50 dark:hover:bg-slate-800/50"
        >
          <span>{{ it.error ? "❌" : it.skipped ? "⏭️" : "✅" }}</span>
          <span class="min-w-0 flex-1 truncate text-sm" :title="it.name">{{ it.name }}</span>
          <template v-if="it.saved != null && !it.error">
            <span class="text-xs text-slate-400">{{ t("original") }}: {{ formatBytes(filesStore.files.find((f) => f.id === it.id)?.size) }}</span>
            <span class="text-xs text-slate-400">→ {{ t("processed") }}: {{ formatBytes(it.output_size) }}</span>
            <span class="w-16 text-right text-xs font-medium text-emerald-600 dark:text-emerald-400">-{{ formatBytes(it.saved) }}</span>
          </template>
          <span v-else-if="it.error" class="max-w-48 truncate text-xs text-rose-500" :title="it.error">{{ it.error }}</span>
          <span v-else class="text-xs text-amber-500">{{ it.error ?? t("skipped") }}</span>
        </div>
      </div>

      <div v-if="exportedMsg" class="mt-4 rounded-lg bg-emerald-50 px-3 py-2 text-sm text-emerald-700 dark:bg-emerald-950/50 dark:text-emerald-300">
        {{ exportedMsg }}
      </div>

      <div class="mt-6 flex flex-wrap items-center justify-between gap-3">
        <label class="flex items-center gap-2 text-sm text-slate-600 dark:text-slate-300">
          <input v-model="replaceOriginal" type="checkbox" class="h-4 w-4 rounded" />
          {{ t("replace_original") }}
        </label>
        <div class="flex gap-2">
          <Button variant="secondary" @click="goHome">{{ t("continue_process") }}</Button>
          <Button v-if="(summary?.failed ?? 0) > 0" variant="outline" @click="retryFailed">{{ t("retry") }}</Button>
          <Button size="lg" @click="exportFiles">📦 {{ t("export") }}</Button>
        </div>
      </div>
    </div>
  </div>
</template>
