export type MediaKind = "image" | "video" | "unsupported";
export type TemplateKind = "slim" | "convert" | "social";

export interface FileInfo {
  id: string;
  path: string;
  name: string;
  size: number;
  kind: MediaKind;
  format: string;
  width?: number | null;
  height?: number | null;
  duration_secs?: number | null;
  thumb?: string | null;
}

export interface WatermarkConfig {
  text?: string | null;
  image?: string | null;
  position: string;
  opacity: number;
}

export interface Template {
  id: string;
  name: string;
  icon: string;
  description: string;
  kind: TemplateKind;
  target_format?: string | null;
  quality?: number | null;
  max_width?: number | null;
  max_height?: number | null;
  video_codec?: string | null;
  video_crf?: number | null;
  video_max_dim?: number | null;
  strip_audio?: boolean | null;
  watermark?: WatermarkConfig | null;
  builtin: boolean;
}

export interface ProcessedItem {
  id: string;
  name: string;
  output_path?: string | null;
  output_size?: number | null;
  saved?: number | null;
  skipped: boolean;
  error?: string | null;
}

export interface BatchSummary {
  total: number;
  succeeded: number;
  failed: number;
  skipped: number;
  saved_bytes: number;
  items: ProcessedItem[];
}

export interface ExportResult {
  exported: number;
  errors: [string, string][];
}

export type ItemStatus = "waiting" | "processing" | "done" | "failed" | "skipped";

export interface TaskItem {
  id: string;
  name: string;
  status: ItemStatus;
  percent: number;
  error?: string | null;
  saved?: number | null;
  new_size?: number | null;
  output_path?: string | null;
}
