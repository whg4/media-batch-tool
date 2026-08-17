//! End-to-end tests for the batch queue (mixed media, failure isolation, cancel).
#![cfg(test)]
use crate::models::{FileInfo, MediaKind, Template, TemplateKind};
use crate::queue::run_batch;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::Listener;

fn temp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("mbt_queue_{tag}_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn make_image(path: &Path, w: u32, h: u32, color: u8) {
    let img = image::RgbaImage::from_fn(w, h, |x, y| {
        image::Rgba([(x % 256) as u8 ^ color, (y % 256) as u8, 128, 255])
    });
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if matches!(ext, "jpg" | "jpeg") {
        image::DynamicImage::ImageRgba8(img).to_rgb8().save(path).unwrap();
    } else {
        img.save(path).unwrap();
    }
}

fn make_video(path: &Path) -> bool {
    let ffmpeg = crate::ffmpeg::find_ffmpeg().unwrap();
    std::process::Command::new(ffmpeg)
        .args([
            "-y", "-f", "lavfi", "-i", "testsrc=size=160x90:rate=5",
            "-t", "1", "-c:v", "libx264", "-pix_fmt", "yuv420p",
        ])
        .arg(path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn file_info(path: &Path, id: &str, kind: MediaKind) -> FileInfo {
    FileInfo {
        id: id.into(),
        path: path.to_string_lossy().into_owned(),
        name: path.file_name().unwrap().to_string_lossy().into_owned(),
        size: std::fs::metadata(path).map(|m| m.len()).unwrap_or(0),
        kind,
        format: path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string(),
        width: None,
        height: None,
        duration_secs: None,
        thumb: None,
    }
}

fn slim_template() -> Template {
    Template {
        id: "slim".into(),
        name: "slim".into(),
        icon: "".into(),
        description: "".into(),
        kind: TemplateKind::Slim,
        target_format: None,
        quality: Some(55),
        max_width: None,
        max_height: None,
        video_codec: Some("h264".into()),
        video_crf: Some(30),
        video_max_dim: Some(120),
        strip_audio: Some(true),
        watermark: None,
        builtin: true,
    }
}

#[tokio::test]
async fn mixed_media_batch_succeeds_and_emits_events() {
    let dir = temp_dir("mixed");
    let img1 = dir.join("a.jpg");
    let img2 = dir.join("b.png");
    make_image(&img1, 1200, 900, 0);
    make_image(&img2, 800, 600, 1);
    let mut files = vec![
        file_info(&img1, "i1", MediaKind::Image),
        file_info(&img2, "i2", MediaKind::Image),
    ];
    let vid = dir.join("v.mp4");
    if make_video(&vid) {
        files.push(file_info(&vid, "v1", MediaKind::Video));
    }

    let app = tauri::test::mock_app().handle().clone();
    let (tx, rx) = mpsc::channel::<serde_json::Value>();
    let _handler = app.listen("batch_complete", move |e| {
        let _ = tx.send(serde_json::from_str(e.payload()).unwrap_or_default());
    });

    let outputs: crate::queue::OutputsMap = Arc::new(Mutex::new(std::collections::HashMap::new()));
    let out_dir = dir.join("out");
    std::fs::create_dir_all(&out_dir).unwrap();

    let summary = run_batch(
        app,
        files,
        slim_template(),
        out_dir,
        Arc::new(AtomicBool::new(false)),
        outputs.clone(),
    )
    .await;

    let total = if summary.total == 3 { 3 } else { 2 };
    assert_eq!(summary.succeeded + summary.skipped + summary.failed, total);
    // every image must either be compressed (smaller) or skipped (never larger)
    assert!(summary.succeeded > 0, "expected at least one success, got {summary:?}");
    assert!(summary.failed == 0, "expected no failures, got {summary:?}");

    // outputs map populated for succeeded items
    let outputs_guard = outputs.lock().unwrap();
    let done = outputs_guard.values().filter(|i| i.output_path.is_some()).count();
    assert_eq!(done, summary.succeeded);
    drop(outputs_guard);

    // batch_complete event was emitted
    let ev = rx
        .recv_timeout(Duration::from_secs(10))
        .expect("batch_complete event must be emitted");
    assert!(ev.get("total").is_some());
    assert!(ev.get("saved_bytes").is_some());

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn failure_isolation_keeps_batch_going() {
    let dir = temp_dir("failiso");
    let good = dir.join("good.jpg");
    make_image(&good, 600, 400, 9);
    let bad = dir.join("corrupt.jpg");
    std::fs::write(&bad, b"this is not a real jpeg image at all").unwrap();

    let files = vec![
        file_info(&good, "g1", MediaKind::Image),
        file_info(&bad, "b1", MediaKind::Image),
    ];
    let app = tauri::test::mock_app().handle().clone();
    let outputs: crate::queue::OutputsMap = Arc::new(Mutex::new(std::collections::HashMap::new()));
    let out_dir = dir.join("out");
    std::fs::create_dir_all(&out_dir).unwrap();

    let summary = run_batch(
        app,
        files,
        slim_template(),
        out_dir,
        Arc::new(AtomicBool::new(false)),
        outputs.clone(),
    )
    .await;

    assert_eq!(summary.failed, 1, "corrupt file must fail");
    assert_eq!(summary.succeeded, 1, "good file must succeed");
    let bad_item = summary.items.iter().find(|i| i.id == "b1").unwrap();
    assert!(bad_item.error.is_some(), "failed item must carry an error");
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn cancel_stops_processing() {
    let dir = temp_dir("cancel");
    let img = dir.join("c.jpg");
    make_image(&img, 400, 300, 7);
    let files = vec![file_info(&img, "c1", MediaKind::Image)];
    let app = tauri::test::mock_app().handle().clone();
    let outputs: crate::queue::OutputsMap = Arc::new(Mutex::new(std::collections::HashMap::new()));
    let out_dir = dir.join("out");
    std::fs::create_dir_all(&out_dir).unwrap();

    let cancel = Arc::new(AtomicBool::new(true)); // cancelled before start
    let summary = run_batch(
        app,
        files,
        slim_template(),
        out_dir,
        cancel,
        outputs.clone(),
    )
    .await;

    assert_eq!(summary.total, 1);
    assert_eq!(summary.failed, 1, "cancelled item should be reported as failed");
    let item = &summary.items[0];
    assert_eq!(item.error.as_deref(), Some("已取消"));
    assert!(item.output_path.is_none());
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
#[ignore = "release benchmark: cargo test --release -- --ignored batch_200_photos"]
async fn batch_200_photos_through_queue() {
    let dir = temp_dir("bench200");
    let mut files = Vec::new();
    for i in 0..200 {
        let p = dir.join(format!("photo_{i}.jpg"));
        make_image(&p, 2000, 1500, (i % 250) as u8);
        files.push(file_info(&p, &format!("p{i}"), MediaKind::Image));
    }
    let app = tauri::test::mock_app().handle().clone();
    let outputs: crate::queue::OutputsMap = Arc::new(Mutex::new(std::collections::HashMap::new()));
    let out_dir = dir.join("out");
    std::fs::create_dir_all(&out_dir).unwrap();

    let start = std::time::Instant::now();
    let summary = run_batch(
        app,
        files,
        slim_template(),
        out_dir,
        Arc::new(AtomicBool::new(false)),
        outputs.clone(),
    )
    .await;
    let elapsed = start.elapsed();

    eprintln!(
        "200 photos (2000x1500) through queue: {:?} | succeeded={} skipped={} failed={} saved={} bytes",
        elapsed, summary.succeeded, summary.skipped, summary.failed, summary.saved_bytes
    );
    assert_eq!(summary.failed, 0, "no file should fail");
    assert_eq!(summary.succeeded + summary.skipped, 200);
    assert!(
        elapsed.as_secs() < 60,
        "200-photo batch too slow for a consumer tool: {elapsed:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn unwritable_output_dir_reports_error() {
    let dir = temp_dir("readonly");
    let img = dir.join("r.jpg");
    make_image(&img, 300, 200, 3);
    let out_dir = dir.join("out");
    std::fs::create_dir_all(&out_dir).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&out_dir, std::fs::Permissions::from_mode(0o555)).unwrap();
    }
    let files = vec![file_info(&img, "r1", MediaKind::Image)];
    let app = tauri::test::mock_app().handle().clone();
    let outputs: crate::queue::OutputsMap = Arc::new(Mutex::new(std::collections::HashMap::new()));
    let summary = run_batch(
        app,
        files,
        slim_template(),
        out_dir.clone(),
        Arc::new(AtomicBool::new(false)),
        outputs.clone(),
    )
    .await;

    assert_eq!(summary.failed, 1, "unwritable output dir must produce a per-file error");
    let item = &summary.items[0];
    assert!(item.error.is_some(), "error must be reported to the UI");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&out_dir, std::fs::Permissions::from_mode(0o755));
    }
    let _ = std::fs::remove_dir_all(&dir);
}
