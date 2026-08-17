use crate::models::MediaKind;
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

/// Generate a small JPEG thumbnail (max 320px) for an image or video file.
pub fn make_thumbnail(
    kind: &MediaKind,
    input: &Path,
    out_dir: &Path,
    file_id: &str,
) -> Result<PathBuf> {
    let out = out_dir.join(format!("{file_id}.jpg"));
    match kind {
        MediaKind::Image => {
            let img = crate::image_proc::load_image(input)?;
            let thumb = img.thumbnail(320, 320);
            let mut buf = std::io::BufWriter::new(std::fs::File::create(&out)?);
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 70)
                .encode_image(&thumb)?;
            Ok(out)
        }
        MediaKind::Video => {
            let ffmpeg = crate::ffmpeg::find_ffmpeg()?;
            let status = std::process::Command::new(ffmpeg)
                .args(["-y", "-ss", "0.5", "-i"])
                .arg(input)
                .args(["-frames:v", "1", "-vf", "scale=320:-2", "-q:v", "5"])
                .arg(&out)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()?;
            if status.success() && out.exists() {
                Ok(out)
            } else {
                Err(anyhow!("无法生成视频缩略图"))
            }
        }
        MediaKind::Unsupported => Err(anyhow!("不支持的媒体类型")),
    }
}
