import { invoke } from "@tauri-apps/api/core";
import type { ExportResult, FileInfo, Template } from "../types";

export const api = {
  analyzeFiles: (paths: string[]) => invoke<FileInfo[]>("analyze_files", { paths }),
  getTemplates: () => invoke<Template[]>("get_templates"),
  saveCustomTemplate: (template: Template) =>
    invoke<Template[]>("save_custom_template", { template }),
  deleteCustomTemplate: (id: string) => invoke<Template[]>("delete_custom_template", { id }),
  startBatch: (fileIds: string[], templateId: string) =>
    invoke<void>("start_batch", { fileIds, templateId }),
  cancelBatch: () => invoke<void>("cancel_batch"),
  exportFiles: (fileIds: string[], targetDir: string, overwrite: boolean) =>
    invoke<ExportResult>("export_files", { fileIds, targetDir, overwrite }),
  pickFolder: () => invoke<string | null>("pick_folder"),
  getAppVersion: () => invoke<string>("get_app_version"),
};
