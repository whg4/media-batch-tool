import { ref } from "vue";

type Dict = Record<string, string>;

const zh: Dict = {
  app_name: "媒体批处理工具",
  privacy_note: "🔒 文件只在本机处理，不上传任何数据",
  mode_slim: "智能瘦身",
  mode_slim_desc: "自动压缩图片与视频，肉眼几乎无差别，体积大幅减小",
  mode_convert: "格式转换",
  mode_convert_desc: "批量转换为 JPG / WebP / AVIF / MP4 等常用格式",
  mode_social: "社交媒体适配",
  mode_social_desc: "按微信、抖音、小红书、Instagram 等平台规格一键适配",
  drop_title: "把图片 / 视频拖到这里",
  drop_sub: "或点击选择文件 / 文件夹",
  drop_hint: "支持拖入整个文件夹，自动识别其中的图片与视频",
  choose_files: "选择文件",
  choose_folder: "选择文件夹",
  clear_all: "清空",
  next_step: "下一步",
  back: "返回",
  unsupported: "不支持",
  choose_template: "选择处理方式",
  template_desc: "选择最接近你需求的方案，参数由应用自动设置",
  custom_template: "自定义模板",
  new_template: "新建模板",
  template_name: "模板名称",
  target_format: "目标格式",
  quality: "质量",
  max_dim: "最大尺寸（像素）",
  video_codec: "视频编码",
  video_crf: "视频质量 (CRF)",
  video_max_dim: "视频最大边长",
  strip_audio: "移除音轨",
  watermark: "水印",
  watermark_text: "水印文字",
  watermark_image: "水印图片",
  watermark_position: "位置",
  watermark_opacity: "不透明度",
  cancel: "取消",
  save: "保存",
  delete: "删除",
  start: "开始处理",
  processing: "处理中",
  cancel_task: "停止",
  processing_progress: "正在处理 {done}/{total}",
  estimated_left: "预计剩余约 1 分钟",
  done_title: "处理完成",
  saved_total: "共节省 {size}",
  original: "原文件",
  processed: "处理后",
  skipped: "已跳过",
  failed: "失败",
  retry: "重试",
  export: "导出到…",
  export_folder: "选择导出文件夹",
  continue_process: "继续处理",
  replace_original: "替换原文件",
  keep_original: "保留原文件",
  exported: "已导出 {n} 个文件",
  empty_hint: "还没有处理记录",
  custom_edit: "编辑自定义模板",
  mode_required: "请先在首页选择处理模式",
  start_processing: "开始处理",
  version: "版本",
  no_templates: "该模式暂无可用的处理方案",
};

const en: Dict = {
  app_name: "Media Batch Tool",
  privacy_note: "🔒 Files never leave your device",
  mode_slim: "Smart Compress",
  mode_slim_desc: "Batch compress images & videos with minimal visible quality loss",
  mode_convert: "Convert",
  mode_convert_desc: "Batch convert to JPG / WebP / AVIF / MP4 and more",
  mode_social: "Social Ready",
  mode_social_desc: "One-click adapt to WeChat, Douyin, Xiaohongshu, Instagram…",
  drop_title: "Drop images / videos here",
  drop_sub: "or click to choose files / folders",
  drop_hint: "Folders are expanded automatically",
  choose_files: "Choose files",
  choose_folder: "Choose folder",
  clear_all: "Clear all",
  next_step: "Next",
  back: "Back",
  unsupported: "Unsupported",
  choose_template: "Choose a recipe",
  template_desc: "Pick the closest use case; parameters are set automatically",
  custom_template: "Custom template",
  new_template: "New template",
  template_name: "Template name",
  target_format: "Target format",
  quality: "Quality",
  max_dim: "Max dimension (px)",
  video_codec: "Video codec",
  video_crf: "Video CRF",
  video_max_dim: "Max video edge",
  strip_audio: "Strip audio",
  watermark: "Watermark",
  watermark_text: "Watermark text",
  watermark_image: "Watermark image",
  watermark_position: "Position",
  watermark_opacity: "Opacity",
  cancel: "Cancel",
  save: "Save",
  delete: "Delete",
  start: "Start",
  processing: "Processing",
  cancel_task: "Stop",
  processing_progress: "Processing {done}/{total}",
  estimated_left: "Est. under 1 minute",
  done_title: "Done",
  saved_total: "Saved {size} in total",
  original: "Before",
  processed: "After",
  skipped: "Skipped",
  failed: "Failed",
  retry: "Retry",
  export: "Export to…",
  export_folder: "Choose export folder",
  continue_process: "Process more",
  replace_original: "Replace originals",
  keep_original: "Keep originals",
  exported: "Exported {n} files",
  empty_hint: "No results yet",
  custom_edit: "Edit custom template",
  mode_required: "Choose a mode on the home screen first",
  start_processing: "Start processing",
  version: "Version",
  no_templates: "No recipes available for this mode",
};

const locale = ref<"zh" | "en">("zh");
const dicts: Record<"zh" | "en", Dict> = { zh, en };

export function setLocale(l: "zh" | "en") {
  locale.value = l;
}

export function t(key: string, vars?: Record<string, string | number>): string {
  let s = dicts[locale.value][key] ?? dicts.zh[key] ?? key;
  if (vars) {
    for (const [k, v] of Object.entries(vars)) {
      s = s.replace(`{${k}}`, String(v));
    }
  }
  return s;
}

export function useLocale() {
  return { locale, setLocale };
}
