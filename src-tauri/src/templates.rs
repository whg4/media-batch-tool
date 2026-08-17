use crate::models::{Template, TemplateKind};
use std::path::PathBuf;

pub fn builtin_templates() -> Vec<Template> {
    vec![
        Template {
            id: "slim-auto".into(),
            name: "智能瘦身".into(),
            icon: "✨".into(),
            description: "自动压缩图片与视频，肉眼几乎无差别，体积大幅减小".into(),
            kind: TemplateKind::Slim,
            target_format: None,
            quality: Some(82),
            max_width: None,
            max_height: None,
            video_codec: Some("h264".into()),
            video_crf: Some(26),
            video_max_dim: Some(1920),
            strip_audio: Some(false),
            watermark: None,
            builtin: true,
        },
        Template {
            id: "convert-jpg".into(),
            name: "转为 JPG".into(),
            icon: "🖼️".into(),
            description: "批量转换为通用 JPG 格式（质量 85%）".into(),
            kind: TemplateKind::Convert,
            target_format: Some("jpg".into()),
            quality: Some(85),
            max_width: None,
            max_height: None,
            video_codec: None,
            video_crf: None,
            video_max_dim: None,
            strip_audio: None,
            watermark: None,
            builtin: true,
        },
        Template {
            id: "convert-webp".into(),
            name: "转为 WebP".into(),
            icon: "🌐".into(),
            description: "转换为 WebP，网页友好、体积小（质量 80%）".into(),
            kind: TemplateKind::Convert,
            target_format: Some("webp".into()),
            quality: Some(80),
            max_width: None,
            max_height: None,
            video_codec: None,
            video_crf: None,
            video_max_dim: None,
            strip_audio: None,
            watermark: None,
            builtin: true,
        },
        Template {
            id: "convert-avif".into(),
            name: "转为 AVIF".into(),
            icon: "🚀".into(),
            description: "转换为 AVIF，新一代高压缩格式（质量 55）".into(),
            kind: TemplateKind::Convert,
            target_format: Some("avif".into()),
            quality: Some(55),
            max_width: None,
            max_height: None,
            video_codec: None,
            video_crf: None,
            video_max_dim: None,
            strip_audio: None,
            watermark: None,
            builtin: true,
        },
        Template {
            id: "convert-mp4".into(),
            name: "转为 MP4".into(),
            icon: "🎬".into(),
            description: "视频统一转为 MP4 (H.264)，兼容性最好".into(),
            kind: TemplateKind::Convert,
            target_format: Some("mp4".into()),
            quality: None,
            max_width: None,
            max_height: None,
            video_codec: Some("h264".into()),
            video_crf: Some(23),
            video_max_dim: None,
            strip_audio: Some(false),
            watermark: None,
            builtin: true,
        },
        Template {
            id: "social-wechat".into(),
            name: "微信发送".into(),
            icon: "💬".into(),
            description: "压缩到微信可发送的大小（图片 ≤25MB，视频 ≤1080p）".into(),
            kind: TemplateKind::Social,
            target_format: None,
            quality: Some(72),
            max_width: Some(1920),
            max_height: Some(1920),
            video_codec: Some("h264".into()),
            video_crf: Some(28),
            video_max_dim: Some(1080),
            strip_audio: Some(false),
            watermark: None,
            builtin: true,
        },
        Template {
            id: "social-douyin".into(),
            name: "抖音".into(),
            icon: "🎵".into(),
            description: "竖屏 1080×1920，MP4 (H.264)，适合发布抖音".into(),
            kind: TemplateKind::Social,
            target_format: Some("mp4".into()),
            quality: None,
            max_width: None,
            max_height: None,
            video_codec: Some("h264".into()),
            video_crf: Some(24),
            video_max_dim: Some(1080),
            strip_audio: Some(false),
            watermark: None,
            builtin: true,
        },
        Template {
            id: "social-moments".into(),
            name: "朋友圈".into(),
            icon: "🟢".into(),
            description: "朋友圈适配：最长边 ≤1440px，JPG 质量 78%".into(),
            kind: TemplateKind::Social,
            target_format: Some("jpg".into()),
            quality: Some(78),
            max_width: Some(1080),
            max_height: Some(1440),
            video_codec: None,
            video_crf: None,
            video_max_dim: None,
            strip_audio: None,
            watermark: None,
            builtin: true,
        },
        Template {
            id: "social-xiaohongshu".into(),
            name: "小红书".into(),
            icon: "📕".into(),
            description: "3:4 竖图适配（宽 ≤1080px，JPG 质量 80%）".into(),
            kind: TemplateKind::Social,
            target_format: Some("jpg".into()),
            quality: Some(80),
            max_width: Some(1080),
            max_height: Some(1440),
            video_codec: None,
            video_crf: None,
            video_max_dim: None,
            strip_audio: None,
            watermark: None,
            builtin: true,
        },
        Template {
            id: "social-instagram".into(),
            name: "Instagram".into(),
            icon: "📸".into(),
            description: "方形/竖图适配（最长边 ≤1080px，JPG 质量 82%）".into(),
            kind: TemplateKind::Social,
            target_format: Some("jpg".into()),
            quality: Some(82),
            max_width: Some(1080),
            max_height: Some(1350),
            video_codec: None,
            video_crf: None,
            video_max_dim: None,
            strip_audio: None,
            watermark: None,
            builtin: true,
        },
    ]
}

pub fn custom_templates_path(app_data: &PathBuf) -> PathBuf {
    app_data.join("custom_templates.json")
}

pub fn load_custom_templates(app_data: &PathBuf) -> Vec<Template> {
    let p = custom_templates_path(app_data);
    std::fs::read_to_string(p)
        .ok()
        .and_then(|s| serde_json::from_str::<Vec<Template>>(&s).ok())
        .unwrap_or_default()
}

pub fn save_custom_templates(app_data: &PathBuf, templates: &[Template]) -> anyhow::Result<()> {
    let p = custom_templates_path(app_data);
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let json = serde_json::to_string_pretty(templates)?;
    std::fs::write(p, json)?;
    Ok(())
}

pub fn all_templates(app_data: &PathBuf) -> Vec<Template> {
    let mut t = builtin_templates();
    t.extend(load_custom_templates(app_data));
    t
}

pub fn template_by_id(app_data: &PathBuf, id: &str) -> Option<Template> {
    all_templates(app_data).into_iter().find(|t| t.id == id)
}

