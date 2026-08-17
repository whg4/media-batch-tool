use crate::image_proc;
use crate::models::{BatchSummary, FileInfo, MediaKind, ProcessedItem, Template};
use crate::video_proc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tauri::Emitter;

pub type OutputsMap = Arc<Mutex<HashMap<String, ProcessedItem>>>;

pub async fn run_batch<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    files: Vec<FileInfo>,
    template: Template,
    out_dir: PathBuf,
    cancel: Arc<AtomicBool>,
    outputs: OutputsMap,
) -> BatchSummary {
    let total = files.len();
    let _ = app.emit(
        "batch_started",
        serde_json::json!({ "total": total, "template_id": template.id }),
    );
    let done = Arc::new(AtomicUsize::new(0));
    let mut results: Vec<(usize, ProcessedItem)> = Vec::with_capacity(total);

    let image_items: Vec<(usize, FileInfo)> = files
        .iter()
        .enumerate()
        .filter(|(_, f)| f.kind == MediaKind::Image)
        .map(|(i, f)| (i, f.clone()))
        .collect();
    let video_items: Vec<(usize, FileInfo)> = files
        .iter()
        .enumerate()
        .filter(|(_, f)| f.kind == MediaKind::Video)
        .map(|(i, f)| (i, f.clone()))
        .collect();

    // ---- images: parallel via rayon ----
    if !image_items.is_empty() {
        let app2 = app.clone();
        let t2 = template.clone();
        let out2 = out_dir.clone();
        let cancel2 = cancel.clone();
        let done2 = done.clone();
        let total2 = total;
        let outputs2 = outputs.clone();
        let par: Vec<(usize, ProcessedItem)> = tokio::task::spawn_blocking(move || {
            use rayon::prelude::*;
            image_items
                .par_iter()
                .map(|(idx, f)| {
                    if cancel2.load(Ordering::Relaxed) {
                        return (
                            *idx,
                            ProcessedItem {
                                id: f.id.clone(),
                                name: f.name.clone(),
                                output_path: None,
                                output_size: None,
                                saved: None,
                                skipped: false,
                                error: Some("已取消".into()),
                            },
                        );
                    }
                    let item = process_one_image(&app2, f, &t2, &out2);
                    outputs2.lock().unwrap().insert(f.id.clone(), item.clone());
                    let d = done2.fetch_add(1, Ordering::SeqCst) + 1;
                    let _ = app2.emit("batch_progress", serde_json::json!({ "done": d, "total": total2 }));
                    (*idx, item)
                })
                .collect()
        })
        .await
        .unwrap_or_default();
        results.extend(par);
    }

    // ---- videos: sequential with concurrency 2 ----
    if !video_items.is_empty() {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(2));
        let mut handles = Vec::new();
        for (idx, f) in video_items {
            let app2 = app.clone();
            let t2 = template.clone();
            let out2 = out_dir.clone();
            let cancel2 = cancel.clone();
            let done2 = done.clone();
            let total2 = total;
            let outputs2 = outputs.clone();
            let sem = semaphore.clone();
            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire_owned().await.unwrap();
                if cancel2.load(Ordering::Relaxed) {
                    return (
                        idx,
                        ProcessedItem {
                            id: f.id.clone(),
                            name: f.name.clone(),
                            output_path: None,
                            output_size: None,
                            saved: None,
                            skipped: false,
                            error: Some("已取消".into()),
                        },
                    );
                }
                let _ = app2.emit(
                    "file_started",
                    serde_json::json!({ "id": f.id, "name": f.name }),
                );
                let path = PathBuf::from(&f.path);
                let mut progress = |p: f64| {
                    let _ = app2.emit(
                        "file_progress",
                        serde_json::json!({ "id": f.id, "percent": (p * 100.0).round() as u64 }),
                    );
                };
                let res = video_proc::process_video(&path, &out2, &t2, &cancel2, &mut progress);
                let item = match res {
                    Ok((out_path, orig, new)) => ProcessedItem {
                        id: f.id.clone(),
                        name: f.name.clone(),
                        output_path: Some(out_path.to_string_lossy().into_owned()),
                        output_size: Some(new),
                        saved: Some(orig.saturating_sub(new)),
                        skipped: false,
                        error: None,
                    },
                    Err(e) => ProcessedItem {
                        id: f.id.clone(),
                        name: f.name.clone(),
                        output_path: None,
                        output_size: None,
                        saved: None,
                        skipped: false,
                        error: Some(e.to_string()),
                    },
                };
                if item.error.is_none() {
                    let _ = app2.emit(
                        "file_completed",
                        serde_json::json!({
                            "id": item.id,
                            "name": item.name,
                            "original_size": f.size,
                            "new_size": item.output_size,
                            "saved": item.saved,
                            "output_path": item.output_path,
                        }),
                    );
                    outputs2.lock().unwrap().insert(f.id.clone(), item.clone());
                } else {
                    let _ = app2.emit(
                        "file_failed",
                        serde_json::json!({ "id": item.id, "name": item.name, "error": item.error }),
                    );
                }
                let d = done2.fetch_add(1, Ordering::SeqCst) + 1;
                let _ = app2.emit("batch_progress", serde_json::json!({ "done": d, "total": total2 }));
                (idx, item)
            }));
        }
        for h in handles {
            if let Ok(r) = h.await {
                results.push(r);
            }
        }
    }

    results.sort_by_key(|(idx, _)| *idx);
    let items: Vec<ProcessedItem> = results.into_iter().map(|(_, i)| i).collect();

    let succeeded = items.iter().filter(|i| i.error.is_none() && !i.skipped).count();
    let skipped = items.iter().filter(|i| i.skipped).count();
    let failed = items.iter().filter(|i| i.error.is_some()).count();
    let saved_bytes: u64 = items.iter().filter_map(|i| i.saved).sum();

    let summary = BatchSummary {
        total,
        succeeded,
        failed,
        skipped,
        saved_bytes,
        items,
    };
    let _ = app.emit("batch_complete", serde_json::to_value(&summary).unwrap_or_default());
    summary
}

fn process_one_image<R: tauri::Runtime>(app: &tauri::AppHandle<R>, file: &FileInfo, template: &Template, out_dir: &Path) -> ProcessedItem {
    let _ = app.emit("file_started", serde_json::json!({ "id": file.id, "name": file.name }));
    let res = image_proc::process_image(Path::new(&file.path), out_dir, template);
    match res {
        Ok(Some(out)) => {
            let item = ProcessedItem {
                id: file.id.clone(),
                name: file.name.clone(),
                output_path: Some(out.out_path.to_string_lossy().into_owned()),
                output_size: Some(out.out_size),
                saved: Some(file.size.saturating_sub(out.out_size)),
                skipped: false,
                error: None,
            };
            let _ = app.emit(
                "file_completed",
                serde_json::json!({
                    "id": item.id,
                    "name": item.name,
                    "original_size": file.size,
                    "new_size": item.output_size,
                    "saved": item.saved,
                    "output_path": item.output_path,
                }),
            );
            item
        }
        Ok(None) => {
            let _ = app.emit(
                "file_skipped",
                serde_json::json!({ "id": file.id, "name": file.name, "reason": "压缩后体积未减小，已跳过" }),
            );
            ProcessedItem {
                id: file.id.clone(),
                name: file.name.clone(),
                output_path: None,
                output_size: None,
                saved: None,
                skipped: true,
                error: None,
            }
        }
        Err(e) => {
            let _ = app.emit(
                "file_failed",
                serde_json::json!({ "id": file.id, "name": file.name, "error": e.to_string() }),
            );
            ProcessedItem {
                id: file.id.clone(),
                name: file.name.clone(),
                output_path: None,
                output_size: None,
                saved: None,
                skipped: false,
                error: Some(e.to_string()),
            }
        }
    }
}
