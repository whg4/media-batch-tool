import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import router from "./router";
import "./assets/main.css";
import { useFilesStore } from "./stores/files";

async function bootstrap() {
  // WDIO test plugin (dev only): enables real-app E2E via embedded WebDriver.
  // Tree-shaken out of production builds (import.meta.env.DEV === false).
  if (import.meta.env.DEV) {
    await import("@wdio/tauri-plugin");
  }

  const pinia = createPinia();
  createApp(App).use(pinia).use(router).mount("#app");

  if (import.meta.env.DEV) {
    // Dev-only hook for real-app E2E: WebDriver cannot automate the native file
    // dialog, so expose a way to seed the files store with real absolute paths.
    (window as unknown as Record<string, unknown>).__MBT_ADD_PATHS__ = (paths: string[]) =>
      useFilesStore(pinia).addPaths(paths);
  }
}

bootstrap();
