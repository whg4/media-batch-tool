<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { useRouter } from "vue-router";
import { ROUTES } from "../router";
import { useModeStore } from "../stores/mode";
import { useFilesStore } from "../stores/files";
import { useTemplateStore } from "../stores/templates";
import { useTaskStore } from "../stores/task";
import { t } from "../locales";
import TemplateCard from "../components/TemplateCard.vue";
import Button from "../components/ui/button.vue";
import Dialog from "../components/ui/dialog.vue";
import type { Template, WatermarkConfig } from "../types";

const router = useRouter();
const modeStore = useModeStore();
const filesStore = useFilesStore();
const templateStore = useTemplateStore();
const taskStore = useTaskStore();

const selectedId = ref<string | null>(null);
const showEditor = ref(false);
const editing = reactive(emptyTemplate());
const editingId = ref<string | null>(null);

const list = computed(() =>
  templateStore.templates.filter((tt) => tt.kind === modeStore.mode),
);
const selected = computed(() =>
  templateStore.templates.find((tt) => tt.id === selectedId.value),
);

function emptyTemplate(): Template {
  return {
    id: "",
    name: "",
    icon: "⚙️",
    description: "自定义处理方案",
    kind: modeStore.mode ?? "convert",
    target_format: "jpg",
    quality: 82,
    max_width: null,
    max_height: null,
    video_codec: "h264",
    video_crf: 26,
    video_max_dim: 1080,
    strip_audio: false,
    watermark: null,
    builtin: false,
  };
}

onMounted(async () => {
  await templateStore.load();
  const first = list.value[0];
  if (first) selectedId.value = first.id;
  else if (modeStore.mode === "slim") selectedId.value = "slim-auto";
});

function openNew() {
  Object.assign(editing, emptyTemplate());
  editingId.value = null;
  showEditor.value = true;
}

function openEdit(tt: Template) {
  Object.assign(editing, JSON.parse(JSON.stringify(tt)));
  editingId.value = tt.id;
  showEditor.value = true;
}

async function saveTemplate() {
  const payload: Template = { ...JSON.parse(JSON.stringify(editing)) };
  if (editingId.value) payload.id = editingId.value;
  await templateStore.save(payload);
  showEditor.value = false;
  selectedId.value = payload.id;
}

async function removeTemplate(tt: Template) {
  await templateStore.remove(tt.id);
  if (selectedId.value === tt.id) selectedId.value = list.value[0]?.id ?? null;
}

async function pickWatermarkImage() {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const sel = await open({ filters: [{ name: "水印图片", extensions: ["png", "jpg", "jpeg", "webp"] }] });
  if (typeof sel === "string") editing.watermark = { ...(editing.watermark ?? defaultWatermark()), image: sel };
}

function defaultWatermark(): WatermarkConfig {
  return { text: null, image: null, position: "bottom-right", opacity: 0.8 };
}

async function start() {
  if (!selected.value) return;
  const ids = filesStore.files.filter((f) => f.kind !== "unsupported").map((f) => f.id);
  await taskStore.start(ids, selected.value.id);
  router.push(ROUTES.processing);
}
</script>

<template>
  <div class="flex h-full flex-col p-6">
    <div class="mb-4 flex items-center justify-between">
      <button class="text-sm text-slate-500 hover:text-slate-800 dark:hover:text-slate-200" @click="router.back()">
        ← {{ t("back") }}
      </button>
      <span class="text-sm font-medium text-slate-400">{{ t("choose_template") }}</span>
      <button class="text-sm text-brand-600 hover:text-brand-700 dark:text-brand-400" @click="openNew">
        + {{ t("new_template") }}
      </button>
    </div>

    <p class="mb-3 text-sm text-slate-500 dark:text-slate-400">{{ t("template_desc") }}</p>

    <div class="grid flex-1 grid-cols-1 content-start gap-3 overflow-y-auto sm:grid-cols-2 lg:grid-cols-3">
      <TemplateCard
        v-for="tt in list"
        :key="tt.id"
        :template="tt"
        :selected="selectedId === tt.id"
        :deletable="!tt.builtin"
        @select="selectedId = tt.id"
        @remove="removeTemplate(tt)"
        @edit="openEdit(tt)"
      />
      <button
        class="flex min-h-32 flex-col items-center justify-center gap-2 rounded-2xl border-2 border-dashed border-slate-300 text-slate-400 transition-colors hover:border-brand-400 hover:text-brand-500 dark:border-slate-700"
        @click="openNew"
      >
        <span class="text-2xl">＋</span>
        <span class="text-sm">{{ t("custom_template") }}</span>
      </button>
    </div>

    <div class="mt-4 flex items-center justify-between">
      <span class="text-sm text-slate-500">{{ filesStore.supportedCount }} 个文件待处理</span>
      <Button size="lg" :disabled="!selected" @click="start">
        {{ t("start_processing") }} →
      </Button>
    </div>

    <Dialog :open="showEditor" :title="t('custom_edit') || t('new_template')" @close="showEditor = false">
      <div class="space-y-4">
        <label class="block">
          <span class="mb-1 block text-xs font-medium text-slate-500">{{ t("template_name") }}</span>
          <input v-model="editing.name" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm dark:border-slate-700 dark:bg-slate-800" placeholder="例如：朋友圈 3:4" />
        </label>

        <div class="grid grid-cols-2 gap-3">
          <label class="block">
            <span class="mb-1 block text-xs font-medium text-slate-500">{{ t("target_format") }}</span>
            <select v-model="editing.target_format" class="w-full rounded-lg border border-slate-300 px-2 py-2 text-sm dark:border-slate-700 dark:bg-slate-800">
              <option value="jpg">JPG</option>
              <option value="png">PNG</option>
              <option value="webp">WebP</option>
              <option value="avif">AVIF</option>
              <option value="mp4">MP4</option>
            </select>
          </label>
          <label class="block">
            <span class="mb-1 block text-xs font-medium text-slate-500">{{ t("quality") }}</span>
            <input v-model.number="editing.quality" type="number" min="1" max="100" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm dark:border-slate-700 dark:bg-slate-800" />
          </label>
        </div>

        <div class="grid grid-cols-3 gap-3">
          <label class="block">
            <span class="mb-1 block text-xs font-medium text-slate-500">{{ t("max_dim") }} W</span>
            <input v-model.number="editing.max_width" type="number" min="0" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm dark:border-slate-700 dark:bg-slate-800" />
          </label>
          <label class="block">
            <span class="mb-1 block text-xs font-medium text-slate-500">{{ t("max_dim") }} H</span>
            <input v-model.number="editing.max_height" type="number" min="0" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm dark:border-slate-700 dark:bg-slate-800" />
          </label>
          <label class="block">
            <span class="mb-1 block text-xs font-medium text-slate-500">{{ t("video_max_dim") }}</span>
            <input v-model.number="editing.video_max_dim" type="number" min="0" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm dark:border-slate-700 dark:bg-slate-800" />
          </label>
        </div>

        <div class="grid grid-cols-2 gap-3">
          <label class="block">
            <span class="mb-1 block text-xs font-medium text-slate-500">{{ t("video_codec") }}</span>
            <select v-model="editing.video_codec" class="w-full rounded-lg border border-slate-300 px-2 py-2 text-sm dark:border-slate-700 dark:bg-slate-800">
              <option value="h264">H.264</option>
              <option value="hevc">H.265 / HEVC</option>
              <option value="av1">AV1</option>
            </select>
          </label>
          <label class="block">
            <span class="mb-1 block text-xs font-medium text-slate-500">{{ t("video_crf") }}</span>
            <input v-model.number="editing.video_crf" type="number" min="0" max="51" class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm dark:border-slate-700 dark:bg-slate-800" />
          </label>
        </div>

        <label class="flex items-center gap-2 text-sm">
          <input v-model="editing.strip_audio" type="checkbox" class="h-4 w-4 rounded" />
          {{ t("strip_audio") }}
        </label>

        <div class="rounded-xl border border-slate-200 p-3 dark:border-slate-700">
          <div class="mb-2 flex items-center justify-between">
            <span class="text-xs font-medium text-slate-500">{{ t("watermark") }}</span>
            <label class="flex items-center gap-1 text-xs">
              <input v-model="editing.watermark" :true-value="editing.watermark ?? defaultWatermark()" :false-value="null" type="checkbox" />
              启用
            </label>
          </div>
          <div v-if="editing.watermark" class="space-y-3">
            <div class="grid grid-cols-2 gap-3">
              <label class="block">
                <span class="mb-1 block text-xs text-slate-500">{{ t("watermark_text") }}</span>
                <input v-model="editing.watermark.text" class="w-full rounded-lg border border-slate-300 px-2 py-1.5 text-sm dark:border-slate-700 dark:bg-slate-800" placeholder="如：@我的账号" />
              </label>
              <div class="flex items-end gap-2">
                <Button variant="secondary" size="sm" @click="pickWatermarkImage">{{ t("watermark_image") }}</Button>
                <span v-if="editing.watermark.image" class="truncate text-xs text-slate-400" :title="editing.watermark.image">已选择</span>
              </div>
            </div>
            <div class="grid grid-cols-2 gap-3">
              <label class="block">
                <span class="mb-1 block text-xs text-slate-500">{{ t("watermark_position") }}</span>
                <select v-model="editing.watermark.position" class="w-full rounded-lg border border-slate-300 px-2 py-1.5 text-sm dark:border-slate-700 dark:bg-slate-800">
                  <option value="top-left">左上</option>
                  <option value="top-right">右上</option>
                  <option value="bottom-left">左下</option>
                  <option value="bottom-right">右下</option>
                  <option value="center">居中</option>
                </select>
              </label>
              <label class="block">
                <span class="mb-1 block text-xs text-slate-500">{{ t("watermark_opacity") }}</span>
                <input v-model.number="editing.watermark.opacity" type="number" min="0.1" max="1" step="0.1" class="w-full rounded-lg border border-slate-300 px-2 py-1.5 text-sm dark:border-slate-700 dark:bg-slate-800" />
              </label>
            </div>
          </div>
        </div>

        <div class="flex justify-end gap-2 pt-1">
          <Button variant="secondary" @click="showEditor = false">{{ t("cancel") }}</Button>
          <Button @click="saveTemplate">{{ t("save") }}</Button>
        </div>
      </div>
    </Dialog>
  </div>
</template>
