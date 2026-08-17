import { defineStore } from "pinia";
import type { TemplateKind } from "../types";

export const useModeStore = defineStore("mode", {
  state: () => ({ mode: null as TemplateKind | null }),
  actions: {
    setMode(m: TemplateKind) {
      this.mode = m;
    },
    clear() {
      this.mode = null;
    },
  },
});
