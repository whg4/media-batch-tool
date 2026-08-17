<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { ROUTES } from "../router";
import { useModeStore } from "../stores/mode";
import ModeCard from "../components/ModeCard.vue";
import { t, useLocale } from "../locales";
import { api } from "../lib/ipc";

const router = useRouter();
const modeStore = useModeStore();
const { locale, setLocale } = useLocale();
const version = ref("");

function select(kind: "slim" | "convert" | "social") {
  modeStore.setMode(kind);
  router.push(ROUTES.drop);
}

onMounted(async () => {
  try {
    version.value = await api.getAppVersion();
  } catch {
    version.value = "dev";
  }
});
</script>

<template>
  <div class="flex flex-1 flex-col items-center justify-center gap-10 p-8">
    <div class="flex items-center gap-2 text-4xl font-bold tracking-tight">
      <span>⚡</span>
      <span>{{ t("app_name") }}</span>
    </div>
    <p class="-mt-6 max-w-md text-center text-sm text-slate-500 dark:text-slate-400">
      图片和视频批量处理，简单到只需要三步：选择场景 → 拖入文件 → 完成
    </p>

    <div class="grid w-full max-w-4xl grid-cols-1 gap-5 md:grid-cols-3">
      <ModeCard
        :icon="'✨'"
        :title="t('mode_slim')"
        :desc="t('mode_slim_desc')"
        kind="slim"
        @select="select"
      />
      <ModeCard
        :icon="'🔄'"
        :title="t('mode_convert')"
        :desc="t('mode_convert_desc')"
        kind="convert"
        @select="select"
      />
      <ModeCard
        :icon="'📱'"
        :title="t('mode_social')"
        :desc="t('mode_social_desc')"
        kind="social"
        @select="select"
      />
    </div>

    <div class="flex flex-col items-center gap-2">
      <span class="rounded-full bg-emerald-50 px-4 py-1.5 text-sm font-medium text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300">
        {{ t("privacy_note") }}
      </span>
      <span class="text-xs text-slate-400">{{ t("version") }} {{ version }}</span>
    </div>

    <button
      class="rounded-lg px-3 py-1 text-xs text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800"
      @click="locale === 'zh' ? setLocale('en') : setLocale('zh')"
    >
      {{ locale === "zh" ? "EN" : "中文" }}
    </button>
  </div>
</template>
