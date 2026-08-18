// In-browser mock of the Tauri v2 IPC surface, injected via addInitScript.
// Provides invoke + event listeners so the Vue app can run in a plain browser.
export const mockTauriInit = () => {
  const listeners: Record<string, Array<(payload: unknown) => void>> = {};
  const callbacks = new Map<number, (payload: unknown) => void>();
  let cbCounter = 0;

  const emit = (event: string, payload: unknown) => {
    for (const h of listeners[event] ?? []) h(payload);
  };
  (window as any).__mbtTest = { emit };

  const files = [
    { id: "f1", path: "/fake/photo1.jpg", name: "photo1.jpg", size: 4_500_000, kind: "image", format: "jpg", width: 4000, height: 3000, duration_secs: null, thumb: null },
    { id: "f2", path: "/fake/photo2.png", name: "photo2.png", size: 8_200_000, kind: "image", format: "png", width: 3000, height: 4000, duration_secs: null, thumb: null },
    { id: "f3", path: "/fake/video1.mp4", name: "video1.mp4", size: 120_000_000, kind: "video", format: "mp4", width: 1920, height: 1080, duration_secs: 65, thumb: null },
  ];

  const templates = [
    { id: "slim-auto", name: "智能瘦身", icon: "✨", description: "自动压缩", kind: "slim", target_format: null, quality: 82, max_width: null, max_height: null, video_codec: "h264", video_crf: 26, video_max_dim: 1920, strip_audio: false, watermark: null, builtin: true },
    { id: "convert-jpg", name: "转为 JPG", icon: "🖼️", description: "转 JPG", kind: "convert", target_format: "jpg", quality: 85, max_width: null, max_height: null, video_codec: null, video_crf: null, video_max_dim: null, strip_audio: null, watermark: null, builtin: true },
    { id: "social-douyin", name: "抖音", icon: "🎵", description: "竖屏 1080×1920", kind: "social", target_format: "mp4", quality: null, max_width: null, max_height: null, video_codec: "h264", video_crf: 24, video_max_dim: 1080, strip_audio: false, watermark: null, builtin: true },
  ];

  const invoke = async (cmd: string, args: Record<string, unknown> = {}) => {
    switch (cmd) {
      case "get_app_version":
        return "0.1.0-e2e";
      case "get_templates":
        return templates;
      case "analyze_files":
        return files;
      case "plugin:dialog|open":
        return ["/fake/photo1.jpg", "/fake/photo2.png", "/fake/video1.mp4"];
      case "plugin:dialog|pick_folder":
        return "/tmp/e2e-out";
      case "pick_folder":
        return "/tmp/e2e-out";
      case "export_files": {
        const ids = (args.fileIds as string[]) ?? [];
        return { exported: ids.length, errors: [] };
      }
      case "cancel_batch":
      case "plugin:event|unlisten":
        return null;
      case "plugin:event|listen": {
        const event = args.event as string;
        const handlerId = args.handler as number;
        (listeners[event] ??= []).push((payload) => {
          const cb = callbacks.get(handlerId);
          if (cb) cb({ event, id: 0, payload });
        });
        return 0;
      }
      case "save_custom_template":
      case "delete_custom_template":
        return templates;
      case "start_batch": {
        // Mirror real backend behaviour: start_batch returns immediately, then
        // progress events stream in asynchronously and batch_complete arrives
        // last — so the Processing screen gets time to render.
        const ids = (args.fileIds as string[]) ?? [];
        const total = ids.length;
        const summary = {
          total, succeeded: total, failed: 0, skipped: 0,
          saved_bytes: 90_000_000,
          items: ids.map((id) => ({ id, name: files.find((f) => f.id === id)?.name ?? id, output_path: `/tmp/e2e-out/${id}.out`, output_size: 1000, saved: 2000, skipped: false, error: null })),
        };
        setTimeout(() => {
          emit("batch_started", { total, template_id: args.templateId });
          let i = 0;
          const timer = setInterval(() => {
            if (i >= ids.length) {
              clearInterval(timer);
              emit("batch_complete", summary);
              return;
            }
            const id = ids[i];
            emit("file_started", { id, name: files.find((f) => f.id === id)?.name ?? id });
            emit("file_progress", { id, percent: 50 });
            const f = files.find((x) => x.id === id);
            const newSize = f ? Math.round(f.size * 0.4) : 1000;
            emit("file_completed", { id, name: f?.name, original_size: f?.size, new_size: newSize, saved: f ? f.size - newSize : 1000, output_path: `/tmp/e2e-out/${id}.out` });
            emit("batch_progress", { done: ++i, total });
          }, 60);
        }, 200);
        return null;
      }
      default:
        throw new Error(`unmocked command: ${cmd}`);
    }
  };

  (window as any).__TAURI_INTERNALS__ = {
    invoke,
    transformCallback: (cb: (...a: unknown[]) => void) => {
      cbCounter += 1;
      callbacks.set(cbCounter, cb as (p: unknown) => void);
      return cbCounter;
    },
    convertFileSrc: (p: string) => `asset://${p}`,
  };
};
