use crate::models::{Template, TemplateKind, WatermarkConfig};
use anyhow::{anyhow, bail, Context, Result};
use image::{DynamicImage, GenericImage, GenericImageView, ImageEncoder, Rgba, RgbaImage};
use ab_glyph::{Font as _, ScaleFont as _};
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub const IMAGE_EXTS: &[&str] = &[
    "jpg", "jpeg", "png", "webp", "gif", "bmp", "tiff", "tif", "avif", "heic", "heif", "ico",
];

pub fn is_image_ext(ext: &str) -> bool {
    let e = ext.to_ascii_lowercase();
    IMAGE_EXTS.contains(&e.as_str())
}

pub fn is_video_ext(ext: &str) -> bool {
    let e = ext.to_ascii_lowercase();
    matches!(
        e.as_str(),
        "mp4" | "mov" | "mkv" | "avi" | "webm" | "flv" | "wmv" | "m4v" | "3gp" | "ts" | "m2ts" | "mts"
    )
}

/// Decode an image with fallbacks (ffmpeg / sips) for exotic formats like HEIC.
pub fn load_image(path: &Path) -> Result<DynamicImage> {
    if let Ok(img) = image::open(path) {
        return Ok(img);
    }
    let tmp = std::env::temp_dir().join(format!("mbt_decode_{}.png", Uuid::new_v4()));
    let mut decoded = false;
    if crate::ffmpeg::convert_to_png(path, &tmp).is_ok() {
        decoded = true;
    }
    #[cfg(target_os = "macos")]
    if !decoded && crate::ffmpeg::sips_to_png(path, &tmp).is_ok() {
        decoded = true;
    }
    if decoded {
        let res = image::open(&tmp);
        let _ = std::fs::remove_file(&tmp);
        if let Ok(img) = res {
            return Ok(img);
        }
    }
    Err(anyhow!("无法解码该图片（格式可能不支持）"))
}

pub struct ImageOutput {
    pub out_path: PathBuf,
    pub out_size: u64,
}

/// Process a single image against a template. Returns None when the file was
/// intentionally skipped (e.g. slim mode produced a larger file).
pub fn process_image(input: &Path, out_dir: &Path, template: &Template) -> Result<Option<ImageOutput>> {
    let original_size = std::fs::metadata(input).context("读取文件信息失败")?.len();
    let src_ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let target_ext = template
        .target_format
        .clone()
        .unwrap_or_else(|| src_ext.clone());
    let out_ext = normalize_ext(&target_ext);

    // Animated GIF: keep as-is (no re-encode support yet)
    if out_ext == "gif" || out_ext == "mp4" {
        return Ok(None);
    }

    let mut img = load_image(input)?;
    apply_resize(&mut img, template);
    if let Some(wm) = &template.watermark {
        apply_watermark(&mut img, wm).context("应用水印失败")?;
    }

    let out_path = out_dir.join(format!("{}.{}", Uuid::new_v4(), out_ext));
    let bytes = encode_image(&img, &out_ext, template.quality)?;
    std::fs::write(&out_path, &bytes)?;
    let new_size = bytes.len() as u64;

    // Slim mode: skip if we could not make it smaller (and format unchanged)
    if template.kind == TemplateKind::Slim && src_ext == out_ext && new_size >= original_size {
        let _ = std::fs::remove_file(&out_path);
        return Ok(None);
    }

    Ok(Some(ImageOutput { out_path, out_size: new_size }))
}

fn normalize_ext(ext: &str) -> String {
    match ext.to_ascii_lowercase().as_str() {
        "jpeg" | "jpg" => "jpg".into(),
        "tiff" | "tif" => "tiff".into(),
        other => other.into(),
    }
}

pub fn encode_image(img: &DynamicImage, ext: &str, quality: Option<u8>) -> Result<Vec<u8>> {
    let q = quality.unwrap_or(85).clamp(1, 100);
    let mut buf: Vec<u8> = Vec::new();
    match ext {
        "jpg" | "jpeg" => {
            let rgb = img.to_rgb8();
            let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, q);
            enc.encode(&rgb, rgb.width(), rgb.height(), image::ExtendedColorType::Rgb8)?;
        }
        "png" => {
            let rgba = img.to_rgba8();
            let enc = image::codecs::png::PngEncoder::new(&mut buf);
            enc.write_image(&rgba, rgba.width(), rgba.height(), image::ExtendedColorType::Rgba8)?;
        }
        "webp" => {
            let enc = webp::Encoder::from_image(img)
                .map_err(|e| anyhow!("WebP 编码失败: {e}"))?;
            buf = enc.encode(q as f32).to_vec();
        }
        "avif" => {
            let tmp_png = std::env::temp_dir().join(format!("mbt_avif_{}.png", Uuid::new_v4()));
            let tmp_avif = std::env::temp_dir().join(format!("mbt_avif_{}.avif", Uuid::new_v4()));
            let rgba = img.to_rgba8();
            {
                let mut f = std::fs::File::create(&tmp_png)?;
                let enc = image::codecs::png::PngEncoder::new(&mut f);
                enc.write_image(&rgba, rgba.width(), rgba.height(), image::ExtendedColorType::Rgba8)?;
            }
            let crf = (60u32.saturating_sub(u32::from(q) / 2)).clamp(20, 45).to_string();
            let ffmpeg = crate::ffmpeg::find_ffmpeg()?;
            let status = std::process::Command::new(ffmpeg)
                .args(["-y", "-i"])
                .arg(&tmp_png)
                .args(["-c:v", "libsvtav1", "-crf"])
                .arg(&crf)
                .args(["-f", "avif"])
                .arg(&tmp_avif)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()?;
            let _ = std::fs::remove_file(&tmp_png);
            if !status.success() || !tmp_avif.exists() {
                let _ = std::fs::remove_file(&tmp_avif);
                bail!("AVIF 编码失败");
            }
            buf = std::fs::read(&tmp_avif)?;
            let _ = std::fs::remove_file(&tmp_avif);
        }
        other => bail!("暂不支持输出格式: {other}"),
    }
    Ok(buf)
}

fn apply_resize(img: &mut DynamicImage, t: &Template) {
    let (w, h) = (img.width(), img.height());
    if w == 0 || h == 0 {
        return;
    }
    match (t.max_width, t.max_height) {
        (Some(mw), Some(mh)) => {
            let scale = (mw as f32 / w as f32).min(mh as f32 / h as f32).min(1.0);
            if scale < 1.0 {
                *img = img.resize(
                    ((w as f32) * scale).round().max(1.0) as u32,
                    ((h as f32) * scale).round().max(1.0) as u32,
                    image::imageops::FilterType::Lanczos3,
                );
            }
        }
        (Some(m), None) | (None, Some(m)) => {
            let longest = w.max(h);
            if longest > m {
                let scale = m as f32 / longest as f32;
                *img = img.resize(
                    ((w as f32) * scale).round().max(1.0) as u32,
                    ((h as f32) * scale).round().max(1.0) as u32,
                    image::imageops::FilterType::Lanczos3,
                );
            }
        }
        _ => {}
    }
}

fn apply_watermark(img: &mut DynamicImage, wm: &WatermarkConfig) -> Result<()> {
    if let Some(overlay) = &wm.image {
        if !overlay.is_empty() {
            let ov = image::open(overlay).context("无法读取水印图片")?;
            let target_w = ((img.width() as f32) * 0.15).round().max(24.0) as u32;
            let target_h = ((ov.height() as f32) * target_w as f32 / ov.width().max(1) as f32).max(1.0) as u32;
            let mut scaled = ov
                .resize(target_w, target_h, image::imageops::FilterType::Lanczos3)
                .to_rgba8();
            apply_opacity(&mut scaled, wm.opacity);
            let (x, y) = position_coords(img.width(), img.height(), scaled.width(), scaled.height(), &wm.position);
            image::imageops::overlay(img, &DynamicImage::ImageRgba8(scaled), i64::from(x), i64::from(y));
        }
    }
    if let Some(text) = &wm.text {
        if !text.is_empty() {
            let font = load_watermark_font()?;
            draw_text_watermark(img, text, &font, wm.opacity, &wm.position)?;
        }
    }
    Ok(())
}

fn apply_opacity(rgba: &mut RgbaImage, opacity: f32) {
    let o = opacity.clamp(0.0, 1.0);
    if (o - 1.0).abs() < f32::EPSILON {
        return;
    }
    for px in rgba.pixels_mut() {
        px[3] = (f32::from(px[3]) * o).round() as u8;
    }
}

fn position_coords(img_w: u32, img_h: u32, el_w: u32, el_h: u32, pos: &str) -> (u32, u32) {
    let margin = 12u32;
    match pos {
        "top-left" => (margin, margin),
        "top-right" => (img_w.saturating_sub(el_w + margin), margin),
        "bottom-left" => (margin, img_h.saturating_sub(el_h + margin)),
        "center" => (
            img_w.saturating_sub(el_w) / 2,
            img_h.saturating_sub(el_h) / 2,
        ),
        _ => (
            img_w.saturating_sub(el_w + margin),
            img_h.saturating_sub(el_h + margin),
        ),
    }
}

fn load_watermark_font() -> Result<ab_glyph::FontArc> {
    let candidates: Vec<PathBuf> = vec![
        // explicit override
        std::env::var_os("MBT_WATERMARK_FONT").map(PathBuf::from),
        #[cfg(target_os = "macos")]
        Some(PathBuf::from("/System/Library/Fonts/PingFang.ttc")),
        #[cfg(target_os = "windows")]
        Some(PathBuf::from("C:\\Windows\\Fonts\\msyh.ttc")),
        Some(PathBuf::from("/System/Library/Fonts/Helvetica.ttc")),
    ]
    .into_iter()
    .flatten()
    .collect();
    for p in candidates {
        if p.exists() {
            if let Ok(bytes) = std::fs::read(&p) {
                if let Ok(font) = ab_glyph::FontArc::try_from_vec(bytes) {
                    return Ok(font);
                }
            }
        }
    }
    Err(anyhow!("未找到可用字体，暂不支持文字水印（可在 MBT_WATERMARK_FONT 指定字体文件）"))
}

fn draw_text_watermark(
    img: &mut DynamicImage,
    text: &str,
    font: &ab_glyph::FontArc,
    opacity: f32,
    position: &str,
) -> Result<()> {
    let scale = ab_glyph::PxScale::from(((img.width().max(img.height()) as f32) / 22.0).clamp(14.0, 96.0));
    let scaled = font.as_scaled(scale);

    // measure
    let mut total_w = 0.0f32;
    let mut max_h = 0.0f32;
    for ch in text.chars() {
        let gid = scaled.glyph_id(ch);
        total_w += scaled.h_advance(gid);
        if let Some(og) = scaled.outline_glyph(gid.with_scale_and_position(scale, ab_glyph::point(0.0, 0.0))) {
            max_h = max_h.max(og.px_bounds().height());
        }
    }
    let text_w = total_w.ceil().max(1.0) as u32;
    let text_h = max_h.ceil().max(1.0) as u32;
    let (x, y) = position_coords(img.width(), img.height(), text_w, text_h, position);
    let baseline_y = y as f32 + scale.y * 0.85;

    let mut draw_pass = |origin_x: u32, baseline_y: f32, color: Rgba<u8>| {
        let mut cursor = origin_x as f32;
        for ch in text.chars() {
            let gid = scaled.glyph_id(ch);
            if let Some(outlined) = scaled.outline_glyph(gid.with_scale_and_position(scale, ab_glyph::point(cursor, baseline_y))) {
                let b = outlined.px_bounds();
                outlined.draw(|gx, gy, cov| {
                    let ax = b.min.x as i64 + i64::from(gx);
                    let ay = b.min.y as i64 + i64::from(gy);
                    if ax >= 0 && ay >= 0 && (ax as u32) < img.width() && (ay as u32) < img.height() {
                        let alpha = (f32::from(color[3]) * cov).round() as u8;
                        if alpha > 0 {
                            let cur = img.get_pixel(ax as u32, ay as u32);
                            img.put_pixel(ax as u32, ay as u32, blend_pixel(cur, color, alpha));
                        }
                    }
                });
            }
            cursor += scaled.h_advance(gid);
        }
    };

    // shadow pass
    draw_pass(x + 2, baseline_y + 2.0, Rgba([0, 0, 0, (150.0 * opacity) as u8]));
    // main pass
    draw_pass(x, baseline_y, Rgba([255, 255, 255, (255.0 * opacity) as u8]));
    Ok(())
}

fn blend_pixel(dst: Rgba<u8>, src: Rgba<u8>, src_alpha: u8) -> Rgba<u8> {
    let a = f32::from(src_alpha) / 255.0;
    let ia = 1.0 - a;
    Rgba([
        (f32::from(src[0]) * a + f32::from(dst[0]) * ia).round() as u8,
        (f32::from(src[1]) * a + f32::from(dst[1]) * ia).round() as u8,
        (f32::from(src[2]) * a + f32::from(dst[2]) * ia).round() as u8,
        f32::from(dst[3]).max(f32::from(src_alpha)).round() as u8,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageBuffer;

    fn test_image() -> DynamicImage {
        let img: RgbaImage = ImageBuffer::from_fn(320, 200, |x, y| {
            Rgba([(x % 256) as u8, (y % 256) as u8, 128, 255])
        });
        DynamicImage::ImageRgba8(img)
    }

    #[test]
    fn encode_jpeg_png_webp() {
        let img = test_image();
        for ext in ["jpg", "png", "webp"] {
            let bytes = encode_image(&img, ext, Some(80)).expect(ext);
            assert!(!bytes.is_empty());
        }
    }

    #[test]
    fn resize_fits_max_box() {
        let mut img = test_image();
        let t = Template {
            max_width: Some(100),
            max_height: Some(100),
            ..default_template()
        };
        apply_resize(&mut img, &t);
        assert!(img.width() <= 100 && img.height() <= 100);
    }

    #[test]
    fn position_mapping() {
        assert_eq!(position_coords(1000, 1000, 100, 50, "top-left"), (12, 12));
        assert_eq!(position_coords(1000, 1000, 100, 50, "top-right"), (1000 - 112, 12));
        assert_eq!(position_coords(1000, 1000, 100, 50, "center"), (450, 475));
    }


    #[test]
    fn process_image_end_to_end_slim() {
        use crate::models::{Template, TemplateKind};
        let dir = std::env::temp_dir().join(format!("mbt_it_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).ok();
        // build a "photo-like" jpeg
        let mut img: RgbaImage = ImageBuffer::new(1200, 800);
        for y in 0..800 {
            for x in 0..1200 {
                img.put_pixel(x, y, Rgba([(x / 4) as u8, (y / 3) as u8, 160, 255]));
            }
        }
        let src = dir.join("photo.jpg");
        let rgb = DynamicImage::ImageRgba8(img.clone()).to_rgb8();
        let mut out = std::fs::File::create(&src).unwrap();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 100)
            .encode(&rgb, rgb.width(), rgb.height(), image::ExtendedColorType::Rgb8)
            .unwrap();
        let template = Template {
            id: "slim".into(),
            name: "slim".into(),
            icon: "".into(),
            description: "".into(),
            kind: TemplateKind::Slim,
            target_format: None,
            quality: Some(50),
            max_width: None,
            max_height: None,
            video_codec: None,
            video_crf: None,
            video_max_dim: None,
            strip_audio: None,
            watermark: None,
            builtin: true,
        };
        let res = process_image(&src, &dir, &template).expect("process should succeed");
        let out = res.expect("slim should produce smaller file");
        assert!(out.out_size > 0 && out.out_size < std::fs::metadata(&src).unwrap().len());
        assert!(out.out_path.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn process_image_with_image_watermark() {
        use crate::models::{Template, TemplateKind, WatermarkConfig};
        let dir = std::env::temp_dir().join(format!("mbt_it_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).ok();
        let src = dir.join("base.png");
        let mut img: RgbaImage = ImageBuffer::new(800, 600);
        for y in 0..600 {
            for x in 0..800 {
                img.put_pixel(x, y, Rgba([100, 150, 200, 255]));
            }
        }
        image::DynamicImage::ImageRgba8(img.clone())
            .save(&src)
            .unwrap();
        // watermark overlay
        let wm = dir.join("wm.png");
        let mut w: RgbaImage = ImageBuffer::new(200, 80);
        for y in 0..80 {
            for x in 0..200 {
                w.put_pixel(x, y, Rgba([255, 0, 0, 200]));
            }
        }
        image::DynamicImage::ImageRgba8(w).save(&wm).unwrap();
        let template = Template {
            id: "wm".into(),
            name: "wm".into(),
            icon: "".into(),
            description: "".into(),
            kind: TemplateKind::Convert,
            target_format: Some("png".into()),
            quality: None,
            max_width: None,
            max_height: None,
            video_codec: None,
            video_crf: None,
            video_max_dim: None,
            strip_audio: None,
            watermark: Some(WatermarkConfig {
                text: None,
                image: Some(wm.to_string_lossy().into_owned()),
                position: "bottom-right".into(),
                opacity: 0.8,
            }),
            builtin: true,
        };
        let res = process_image(&src, &dir, &template).expect("watermark process should succeed");
        let out = res.expect("convert should produce output");
        assert!(out.out_path.exists());
        let decoded = image::open(&out.out_path).unwrap();
        // watermark pixels present (bottom-right corner differs from plain color)
        let (x, y) = (decoded.width() - 40, decoded.height() - 30);
        let px = decoded.get_pixel(x, y);
        assert!(px[0] > 180, "watermark red channel should be visible, got {}", px[0]);
        let _ = std::fs::remove_dir_all(&dir);
    }


    #[cfg(target_os = "macos")]
    #[test]
    fn heic_decodes_via_sips_fallback() {
        let dir = std::env::temp_dir().join(format!("mbt_heic_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).ok();
        let png = dir.join("src.png");
        let img: RgbaImage = ImageBuffer::from_fn(320, 240, |x, y| {
            Rgba([(x % 256) as u8, (y % 256) as u8, 180, 255])
        });
        image::DynamicImage::ImageRgba8(img).save(&png).unwrap();
        let heic = dir.join("photo.heic");
        let ok = std::process::Command::new("sips")
            .args(["-s", "format", "heic"])
            .arg(&png)
            .args(["--out"])
            .arg(&heic)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !ok || !heic.exists() {
            return; // sips cannot create HEIC on this system
        }
        let decoded = load_image(&heic).expect("HEIC must decode via the sips fallback");
        assert!(decoded.width() == 320 && decoded.height() == 240);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    #[ignore = "benchmark: run with cargo test --release -- --ignored bench_compression"]
    fn bench_compression() {
        use std::time::Instant;
        // synthetic photo-like image: gradient + noise
        let (w, h) = (2048u32, 1536u32);
        let mut img: RgbaImage = ImageBuffer::new(w, h);
        for y in 0..h {
            for x in 0..w {
                let nx = (x % 97) as u8;
                let ny = (y % 89) as u8;
                img.put_pixel(
                    x,
                    y,
                    Rgba([
                        ((x * 255) / w) as u8 ^ nx,
                        ((y * 255) / h) as u8 ^ ny,
                        (((x + y) * 255) / (w + h)) as u8,
                        255,
                    ]),
                );
            }
        }
        let dynimg = DynamicImage::ImageRgba8(img);
        for (ext, q) in [("jpg", 82), ("webp", 80), ("png", 0)] {
            let start = Instant::now();
            let bytes = encode_image(&dynimg, ext, Some(q)).expect(ext);
            let ms = start.elapsed().as_millis();
            eprintln!("bench {ext} q{q}: {:>8} KB  in {ms:>5} ms", bytes.len() / 1024);
        }
        // AVIF via ffmpeg
        let start = Instant::now();
        let bytes = encode_image(&dynimg, "avif", Some(55)).expect("avif");
        let ms = start.elapsed().as_millis();
        eprintln!("bench avif q55: {:>8} KB  in {ms:>5} ms", bytes.len() / 1024);
        // sanity: all encoders produce output
        assert!(!bytes.is_empty());
    }

    fn default_template() -> Template {
        Template {
            id: "t".into(),
            name: "t".into(),
            icon: "".into(),
            description: "".into(),
            kind: TemplateKind::Convert,
            target_format: None,
            quality: None,
            max_width: None,
            max_height: None,
            video_codec: None,
            video_crf: None,
            video_max_dim: None,
            strip_audio: None,
            watermark: None,
            builtin: true,
        }
    }
}
