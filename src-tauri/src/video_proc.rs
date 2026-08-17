use crate::models::Template;
use anyhow::{bail, Context, Result};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub width: u32,
    pub height: u32,
    pub duration_secs: Option<f64>,
    pub codec: Option<String>,
}

/// Probe video metadata using ffprobe (JSON output).
pub fn probe_video(path: &Path) -> Result<VideoInfo> {
    let ffprobe = crate::ffmpeg::find_ffprobe()?;
    let out = Command::new(ffprobe)
        .args(["-v", "error", "-print_format", "json", "-show_streams", "-show_format"])
        .arg(path)
        .output()
        .context("ffprobe 执行失败")?;
    if !out.status.success() {
        bail!("ffprobe 读取失败");
    }
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).context("ffprobe 输出解析失败")?;
    let streams = json.get("streams").and_then(|s| s.as_array()).cloned().unwrap_or_default();
    let mut info = VideoInfo {
        width: 0,
        height: 0,
        duration_secs: None,
        codec: None,
    };
    for s in &streams {
        if s.get("codec_type").and_then(|c| c.as_str()) == Some("video") {
            info.width = s.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            info.height = s.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            info.codec = s.get("codec_name").and_then(|v| v.as_str()).map(String::from);
        }
    }
    info.duration_secs = json
        .get("format")
        .and_then(|f| f.get("duration"))
        .and_then(|d| d.as_str())
        .and_then(|d| d.parse::<f64>().ok());
    if info.width == 0 || info.height == 0 {
        bail!("无法读取视频尺寸");
    }
    Ok(info)
}

/// Transcode a video according to the template. Returns (output_path, original_size, output_size).
/// `cancel` kills the ffmpeg process when set; `on_progress` receives 0.0-1.0.
pub fn process_video(
    input: &Path,
    out_dir: &Path,
    template: &Template,
    cancel: &AtomicBool,
    on_progress: &mut dyn FnMut(f64),
) -> Result<(PathBuf, u64, u64)> {
    let original_size = std::fs::metadata(input).context("读取文件信息失败")?.len();
    let info = probe_video(input)?;
    let target_ext = template.target_format.clone().unwrap_or_else(|| "mp4".into());
    let out_ext = if target_ext == "webm" { "webm" } else { "mp4" };
    let out_path = out_dir.join(format!("{}.{}", Uuid::new_v4(), out_ext));

    let codec = template.video_codec.as_deref().unwrap_or("h264");
    let (encoder, _fallback_ext): (&str, &str) = match codec {
        "hevc" | "h265" => ("libx265", "mp4"),
        "av1" => ("libsvtav1", "mp4"),
        "vp9" => ("libvpx-vp9", "webm"),
        _ => ("libx264", "mp4"),
    };
    let crf = template.video_crf.unwrap_or(23).clamp(0, 51).to_string();

    let mut args: Vec<String> = vec!["-y".into(), "-i".into(), input.to_string_lossy().into_owned()];

    // scale to fit max dimension
    if let Some(max_dim) = template.video_max_dim {
        if info.width.max(info.height) > max_dim {
            args.push("-vf".into());
            args.push(format!(
                "scale='min(iw,{m})':'min(ih,{m})':force_original_aspect_ratio=decrease",
                m = max_dim
            ));
        }
    }

    args.push("-c:v".into());
    args.push(encoder.into());
    args.push("-crf".into());
    args.push(crf);
    args.push("-preset".into());
    args.push("medium".into());

    // audio: strip or copy
    if template.strip_audio.unwrap_or(false) {
        args.push("-an".into());
    } else {
        args.push("-c:a".into());
        args.push("aac".into());
        args.push("-b:a".into());
        args.push("128k".into());
    }
    args.push("-movflags".into());
    args.push("+faststart".into());
    args.push("-progress".into());
    args.push("pipe:1".into());
    args.push("-nostats".into());
    args.push(out_path.to_string_lossy().into_owned());

    let mut child = Command::new(crate::ffmpeg::find_ffmpeg()?)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("启动 ffmpeg 失败")?;

    let total_us = info.duration_secs.unwrap_or(0.0) * 1_000_000.0;
    let mut progress_ok = true;
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if cancel.load(Ordering::Relaxed) {
                let _ = child.kill();
                progress_ok = false;
                break;
            }
            let Ok(line) = line else { continue };
            if let Some(rest) = line.strip_prefix("out_time_us=") {
                if let Ok(us) = rest.parse::<f64>() {
                    if total_us > 0.0 {
                        on_progress((us / total_us).clamp(0.0, 1.0));
                    }
                }
            } else if line.starts_with("progress=end") {
                on_progress(1.0);
            }
        }
    }
    let status = child.wait()?;
    if cancel.load(Ordering::Relaxed) {
        let _ = std::fs::remove_file(&out_path);
        bail!("已取消");
    }
    if !progress_ok || !status.success() || !out_path.exists() {
        let _ = std::fs::remove_file(&out_path);
        bail!("视频转码失败");
    }
    let output_size = std::fs::metadata(&out_path)?.len();
    Ok((out_path, original_size, output_size))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_video_end_to_end() {
        use crate::models::{Template, TemplateKind};
        use std::sync::atomic::AtomicBool;
        let dir = std::env::temp_dir().join(format!("mbt_vt_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).ok();
        let vid = dir.join("sample.mp4");
        let ffmpeg = crate::ffmpeg::find_ffmpeg().unwrap();
        let ok = Command::new(&ffmpeg)
            .args(["-y", "-f", "lavfi", "-i", "testsrc=size=320x180:rate=10", "-t", "2", "-c:v", "libx264", "-pix_fmt", "yuv420p"])
            .arg(&vid)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !ok {
            return;
        }
        let template = Template {
            id: "v".into(),
            name: "v".into(),
            icon: "".into(),
            description: "".into(),
            kind: TemplateKind::Slim,
            target_format: None,
            quality: None,
            max_width: None,
            max_height: None,
            video_codec: Some("h264".into()),
            video_crf: Some(30),
            video_max_dim: Some(160),
            strip_audio: Some(true),
            watermark: None,
            builtin: true,
        };
        let cancel = AtomicBool::new(false);
        let mut progress: Vec<f64> = Vec::new();
        let (out_path, orig, new) = process_video(&vid, &dir, &template, &cancel, &mut |p| progress.push(p))
            .expect("transcode should succeed");
        assert!(out_path.exists());
        assert!(new > 0);
        let _ = orig;
        // progress should have been reported
        assert!(!progress.is_empty(), "progress callbacks expected");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn probe_generated_video() {
        let dir = std::env::temp_dir().join("mbt_video_test");
        std::fs::create_dir_all(&dir).ok();
        let vid = dir.join("sample.mp4");
        if !vid.exists() {
            // generate a tiny video with ffmpeg
            let ffmpeg = crate::ffmpeg::find_ffmpeg().unwrap();
            let ok = Command::new(ffmpeg)
                .args(["-y", "-f", "lavfi", "-i", "testsrc=size=160x90:rate=5", "-t", "1", "-c:v", "libx264", "-pix_fmt", "yuv420p"])
                .arg(&vid)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if !ok {
                return; // skip if ffmpeg unavailable
            }
        }
        let info = probe_video(&vid).expect("probe should work");
        assert!(info.width > 0 && info.height > 0);
        assert!(info.duration_secs.unwrap_or(0.0) > 0.0);
    }
}
