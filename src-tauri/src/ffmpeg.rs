use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Resolve the ffmpeg/ffprobe executable:
/// 1. env override 2. bundled sidecar next to exe 3. system PATH
pub fn find_ffmpeg() -> Result<PathBuf> {
    find_binary("ffmpeg", "MEDIA_BATCH_FFMPEG")
}
pub fn find_ffprobe() -> Result<PathBuf> {
    find_binary("ffprobe", "MEDIA_BATCH_FFPROBE")
}

fn find_binary(name: &str, env_key: &str) -> Result<PathBuf> {
    if let Ok(p) = std::env::var(env_key) {
        let pb = PathBuf::from(p);
        if pb.exists() {
            return Ok(pb);
        }
    }
    // Bundled sidecar next to the executable (Tauri externalBin names)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let target = current_target_triple();
            for candidate in [
                dir.join(name),
                dir.join(format!("{name}-{target}")),
                dir.join(format!("{name}-{}", target)),
            ] {
                if candidate.exists() {
                    return Ok(candidate);
                }
            }
        }
    }
    if let Some(p) = find_in_path(name) {
        return Ok(p);
    }
    Err(anyhow!("未找到 {name}，请安装 ffmpeg 或在环境变量 {} 中指定路径", env_key))
}

fn current_target_triple() -> String {
    let os = std::env::consts::OS; // macos / windows / linux
    let arch = std::env::consts::ARCH; // aarch64 / x86_64
    match os {
        "macos" => format!("{arch}-apple-darwin"),
        "windows" => format!("{arch}-pc-windows-msvc"),
        _ => format!("{arch}-unknown-linux-gnu"),
    }
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let cand = dir.join(name);
        if cand.exists() {
            return Some(cand);
        }
    }
    None
}

/// Convert any decodable media to a PNG at `out` (used for HEIC / exotic formats).
pub fn convert_to_png(input: &Path, out: &Path) -> Result<()> {
    let ffmpeg = find_ffmpeg()?;
    let status = Command::new(ffmpeg)
        .args(["-y", "-i"])
        .arg(input)
        .args(["-frames:v", "1", "-f", "image2"])
        .arg(out)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;
    if status.success() && out.exists() {
        Ok(())
    } else {
        Err(anyhow!("ffmpeg 无法转换该文件"))
    }
}

/// Use macOS `sips` to convert an image (HEIC etc) to PNG.
#[cfg(target_os = "macos")]
pub fn sips_to_png(input: &Path, out: &Path) -> Result<()> {
    let status = Command::new("sips")
        .args(["-s", "format", "png", "-s", "formatOptions", "default"])
        .arg(input)
        .args(["--out"])
        .arg(out)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;
    if status.success() && out.exists() {
        Ok(())
    } else {
        Err(anyhow!("sips 无法转换该文件"))
    }
}

/// Simple test helper: probe a file with ffprobe and return whether it exists / is media.
#[cfg(test)]
pub fn probe_is_media(path: &Path) -> bool {
    let ffprobe = find_ffprobe().ok();
    match ffprobe {
        Some(p) => Command::new(p)
            .args(["-v", "error", "-show_format", "-of", "json"])
            .arg(path)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false),
        None => false,
    }
}
