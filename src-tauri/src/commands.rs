use crate::image_proc;
use crate::models::{ExportResult, FileInfo, MediaKind, Template};
use crate::templates;
use crate::video_proc;
use crate::AppState;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

#[tauri::command]
pub fn analyze_files(
    app: AppHandle,
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<Vec<FileInfo>, String> {
    let mut files: Vec<FileInfo> = Vec::new();
    for p in &paths {
        let pb = PathBuf::from(p);
        if pb.is_dir() {
            for entry in walkdir::WalkDir::new(&pb)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    if let Some(fi) = build_file_info(entry.path()) {
                        files.push(fi);
                    }
                }
            }
        } else if pb.is_file() {
            if let Some(fi) = build_file_info(&pb) {
                files.push(fi);
            }
        }
    }

    // dedupe by absolute path, keep first
    let mut seen = std::collections::HashSet::new();
    files.retain(|f| seen.insert(f.path.clone()));

    let mut map = state.files.lock().unwrap();
    for f in files.iter_mut() {
        if f.kind != MediaKind::Unsupported {
            let thumb_dir = state.thumb_dir.clone();
            if let Ok(t) =
                crate::thumbnails::make_thumbnail(&f.kind, Path::new(&f.path), &thumb_dir, &f.id)
            {
                f.thumb = Some(t.to_string_lossy().into_owned());
            }
        }
        map.insert(f.id.clone(), f.clone());
    }
    let _ = app;
    Ok(files)
}

fn build_file_info(path: &Path) -> Option<FileInfo> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let kind = if image_proc::is_image_ext(&ext) {
        MediaKind::Image
    } else if image_proc::is_video_ext(&ext) {
        MediaKind::Video
    } else {
        MediaKind::Unsupported
    };
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    let size = std::fs::metadata(path).ok()?.len();
    let mut fi = FileInfo {
        id: Uuid::new_v4().to_string(),
        path: path.to_string_lossy().into_owned(),
        name,
        size,
        kind,
        format: ext,
        width: None,
        height: None,
        duration_secs: None,
        thumb: None,
    };
    match fi.kind {
        MediaKind::Image => {
            if let Ok(img) = image_proc::load_image(path) {
                fi.width = Some(img.width());
                fi.height = Some(img.height());
            }
        }
        MediaKind::Video => {
            if let Ok(info) = video_proc::probe_video(path) {
                fi.width = Some(info.width);
                fi.height = Some(info.height);
                fi.duration_secs = info.duration_secs;
            }
        }
        _ => {}
    }
    Some(fi)
}

#[tauri::command]
pub fn get_templates(state: State<'_, AppState>) -> Vec<Template> {
    templates::all_templates(&state.app_data)
}

#[tauri::command]
pub fn save_custom_template(
    state: State<'_, AppState>,
    mut template: Template,
) -> Result<Vec<Template>, String> {
    if template.name.trim().is_empty() {
        return Err("模板名称不能为空".into());
    }
    let mut customs: Vec<Template> = templates::load_custom_templates(&state.app_data);
    customs.retain(|t| !(template.id.is_empty() || t.id == template.id));
    if template.id.trim().is_empty() {
        template.id = format!("custom-{}", Uuid::new_v4());
    }
    template.builtin = false;
    customs.push(template);
    templates::save_custom_templates(&state.app_data, &customs).map_err(|e| e.to_string())?;
    Ok(templates::all_templates(&state.app_data))
}

#[tauri::command]
pub fn delete_custom_template(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<Template>, String> {
    let mut customs: Vec<Template> = templates::load_custom_templates(&state.app_data);
    customs.retain(|t| t.id != id);
    templates::save_custom_templates(&state.app_data, &customs).map_err(|e| e.to_string())?;
    Ok(templates::all_templates(&state.app_data))
}

#[tauri::command]
pub fn start_batch(
    app: AppHandle,
    state: State<'_, AppState>,
    file_ids: Vec<String>,
    template_id: String,
) -> Result<(), String> {
    {
        let running = state.batch_running.lock().unwrap();
        if *running {
            return Err("已有任务正在运行，请等待完成或先取消".into());
        }
    }
    let template = templates::template_by_id(&state.app_data, &template_id)
        .ok_or_else(|| "模板不存在".to_string())?;
    let files: Vec<FileInfo> = {
        let map = state.files.lock().unwrap();
        file_ids
            .iter()
            .filter_map(|id| map.get(id).cloned())
            .filter(|f| f.kind != MediaKind::Unsupported)
            .collect()
    };
    if files.is_empty() {
        return Err("没有可处理的文件（请确认选择的文件是图片或视频）".into());
    }

    // fresh batch directory; clear stale outputs
    let batch_dir = state.output_root.join(Uuid::new_v4().to_string());
    std::fs::create_dir_all(&batch_dir).map_err(|e| e.to_string())?;
    if let Ok(entries) = std::fs::read_dir(&state.output_root) {
        for e in entries.flatten() {
            if e.path() != batch_dir {
                let _ = std::fs::remove_dir_all(e.path());
            }
        }
    }
    state.outputs.lock().unwrap().clear();

    let cancel = Arc::new(std::sync::atomic::AtomicBool::new(false));
    *state.cancel.lock().unwrap() = Some(cancel.clone());
    *state.batch_running.lock().unwrap() = true;

    let app2 = app.clone();
    let outputs = state.outputs.clone();
    tauri::async_runtime::spawn(async move {
        let summary = crate::queue::run_batch(app2.clone(), files, template, batch_dir, cancel, outputs).await;
        let _ = summary;
        if let Some(st) = app2.try_state::<AppState>() {
            *st.batch_running.lock().unwrap() = false;
            *st.cancel.lock().unwrap() = None;
        }
    });
    Ok(())
}

#[tauri::command]
pub fn cancel_batch(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(c) = state.cancel.lock().unwrap().as_ref() {
        c.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[tauri::command]
pub fn export_files(
    state: State<'_, AppState>,
    file_ids: Vec<String>,
    target_dir: String,
    overwrite: bool,
) -> Result<ExportResult, String> {
    let dir = PathBuf::from(&target_dir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("无法创建目标目录: {e}"))?;
    let outputs = state.outputs.lock().unwrap();
    let mut exported = 0usize;
    let mut errors: Vec<(String, String)> = Vec::new();
    for id in &file_ids {
        let Some(item) = outputs.get(id) else {
            errors.push((id.clone(), "未找到处理结果".into()));
            continue;
        };
        let Some(out_path) = &item.output_path else {
            continue; // skipped / failed -> no output
        };
        let op = PathBuf::from(out_path);
        let ext = op
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();
        let stem = Path::new(&item.name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output")
            .to_string();
        let mut dest = dir.join(format!("{stem}.{ext}"));
        if dest.exists() && !overwrite {
            let mut i = 1;
            loop {
                dest = dir.join(format!("{stem} ({i}).{ext}"));
                if !dest.exists() {
                    break;
                }
                i += 1;
            }
        }
        match std::fs::copy(&op, &dest) {
            Ok(_) => exported += 1,
            Err(e) => errors.push((item.name.clone(), e.to_string())),
        }
    }
    Ok(ExportResult { exported, errors })
}

#[tauri::command]
pub async fn pick_folder(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = tauri::async_runtime::spawn_blocking(move || {
        app.dialog().file().blocking_pick_folder()
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(picked.map(|p| p.to_string()))
}

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").into()
}
