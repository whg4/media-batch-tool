import { ref } from "vue";
import { useFilesStore } from "../stores/files";

export function useFileDrop() {
  const filesStore = useFilesStore();
  const dragging = ref(false);
  let depth = 0;

  function onDragEnter(e: DragEvent) {
    e.preventDefault();
    depth += 1;
    dragging.value = true;
  }
  function onDragOver(e: DragEvent) {
    e.preventDefault();
  }
  function onDragLeave(e: DragEvent) {
    e.preventDefault();
    depth -= 1;
    if (depth <= 0) {
      depth = 0;
      dragging.value = false;
    }
  }
  async function onDrop(e: DragEvent) {
    e.preventDefault();
    depth = 0;
    dragging.value = false;
    const paths: string[] = [];
    // In the webview, dropped files expose a `path` on the File object (Tauri).
    for (const f of Array.from(e.dataTransfer?.files ?? [])) {
      const p = (f as unknown as { path?: string }).path;
      if (p) paths.push(p);
    }
    if (paths.length > 0) await filesStore.addPaths(paths);
  }

  return { dragging, onDragEnter, onDragOver, onDragLeave, onDrop };
}
