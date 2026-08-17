import { defineStore } from "pinia";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { api } from "../lib/ipc";
import type { BatchSummary, TaskItem } from "../types";

export const useTaskStore = defineStore("task", {
  state: () => ({
    items: [] as TaskItem[],
    running: false,
    done: 0,
    total: 0,
    summary: null as BatchSummary | null,
    lastTemplateId: "",
    error: null as string | null,
    _unlisteners: [] as UnlistenFn[],
  }),
  getters: {
    percent(state): number {
      if (state.total === 0) return 0;
      // weighted by per-file percent for smoother UI
      const sum = state.items.reduce((s, it) => s + it.percent, 0);
      const total = state.items.length || 1;
      return Math.min(100, Math.round(sum / total));
    },
  },
  actions: {
    async start(fileIds: string[], templateId: string) {
      this.items = fileIds.map((id) => ({
        id,
        name: "",
        status: "waiting",
        percent: 0,
      }));
      this.done = 0;
      this.total = fileIds.length;
      this.summary = null;
      this.lastTemplateId = templateId;
      this.error = null;
      this.running = true;
      await this.bindEvents();
      try {
        await api.startBatch(fileIds, templateId);
      } catch (e) {
        this.error = String(e);
        this.running = false;
      }
    },
    async bindEvents() {
      await this.unbindEvents();
      this._unlisteners.push(
        await listen<{ id: string; name: string }>("file_started", (e) => {
          const it = this.item(e.payload.id);
          if (it) {
            it.name = e.payload.name;
            it.status = "processing";
            it.percent = 1;
          }
        }),
        await listen<{ id: string; percent: number }>("file_progress", (e) => {
          const it = this.item(e.payload.id);
          if (it) it.percent = Math.max(it.percent, e.payload.percent);
        }),
        await listen<{ id: string; name: string; saved: number; new_size: number; output_path: string }>(
          "file_completed",
          (e) => {
            const it = this.item(e.payload.id);
            if (it) {
              it.status = "done";
              it.percent = 100;
              it.saved = e.payload.saved;
              it.new_size = e.payload.new_size;
              it.output_path = e.payload.output_path;
              it.name = e.payload.name;
            }
            this.done += 1;
          },
        ),
        await listen<{ id: string; name: string; reason: string }>("file_skipped", (e) => {
          const it = this.item(e.payload.id);
          if (it) {
            it.status = "skipped";
            it.percent = 100;
            it.name = e.payload.name;
            it.error = e.payload.reason;
          }
          this.done += 1;
        }),
        await listen<{ id: string; name: string; error: string }>("file_failed", (e) => {
          const it = this.item(e.payload.id);
          if (it) {
            it.status = "failed";
            it.percent = 100;
            it.error = e.payload.error;
            it.name = e.payload.name;
          }
          this.done += 1;
        }),
        await listen<{ done: number; total: number }>("batch_progress", (e) => {
          this.done = e.payload.done;
          this.total = e.payload.total;
        }),
        await listen<BatchSummary>("batch_complete", (e) => {
          this.summary = e.payload;
          this.running = false;
          this.done = e.payload.total;
        }),
      );
    },
    async unbindEvents() {
      for (const u of this._unlisteners) u();
      this._unlisteners = [];
    },
    async cancel() {
      try {
        await api.cancelBatch();
      } catch (e) {
        this.error = String(e);
      }
    },
    item(id: string): TaskItem | undefined {
      return this.items.find((it) => it.id === id);
    },
    reset() {
      this.items = [];
      this.running = false;
      this.done = 0;
      this.total = 0;
      this.summary = null;
      this.error = null;
    },
  },
});
