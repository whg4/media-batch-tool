import { defineStore } from "pinia";
import { api } from "../lib/ipc";
import type { Template } from "../types";

export const useTemplateStore = defineStore("templates", {
  state: () => ({ templates: [] as Template[], loaded: false }),
  getters: {
    byKind: (state) => (kind: string) => state.templates.filter((t) => t.kind === kind),
  },
  actions: {
    async load() {
      this.templates = await api.getTemplates();
      this.loaded = true;
    },
    async save(t: Template) {
      this.templates = await api.saveCustomTemplate(t);
    },
    async remove(id: string) {
      this.templates = await api.deleteCustomTemplate(id);
    },
  },
});
