import { defineStore } from "pinia";
import { api } from "../lib/ipc";
import type { FileInfo } from "../types";

export const useFilesStore = defineStore("files", {
  state: () => ({
    files: [] as FileInfo[],
    analyzing: false,
    error: null as string | null,
  }),
  getters: {
    totalSize(state): number {
      return state.files.reduce((s, f) => s + f.size, 0);
    },
    supportedCount(state): number {
      return state.files.filter((f) => f.kind !== "unsupported").length;
    },
    unsupportedCount(state): number {
      return state.files.filter((f) => f.kind === "unsupported").length;
    },
  },
  actions: {
    async addPaths(paths: string[]) {
      this.analyzing = true;
      this.error = null;
      try {
        const added = await api.analyzeFiles(paths);
        const existing = new Set(this.files.map((f) => f.path));
        this.files.push(...added.filter((f) => !existing.has(f.path)));
      } catch (e) {
        this.error = String(e);
      } finally {
        this.analyzing = false;
      }
    },
    clear() {
      this.files = [];
      this.error = null;
    },
    remove(id: string) {
      this.files = this.files.filter((f) => f.id !== id);
    },
  },
});
