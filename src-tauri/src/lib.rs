#[cfg(test)]
mod queue_tests;
#[cfg(test)]
mod updater_sig_test;
mod commands;
mod ffmpeg;
mod image_proc;
mod models;
mod queue;
mod templates;
mod thumbnails;
mod video_proc;

use models::FileInfo;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub struct AppState {
    pub files: Mutex<HashMap<String, FileInfo>>,
    pub outputs: queue::OutputsMap,
    pub batch_running: Mutex<bool>,
    pub cancel: Mutex<Option<Arc<AtomicBool>>>,
    pub thumb_dir: PathBuf,
    pub output_root: PathBuf,
    pub app_data: PathBuf,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build());
    #[cfg(feature = "wdio")]
    let builder = builder
        .plugin(tauri_plugin_wdio::init())
        .plugin(tauri_plugin_wdio_webdriver::init());
    builder
        .setup(|app| {
            let app_data = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data)?;
            let thumb_dir = app_data.join("thumbs");
            let output_root = app_data.join("batches");
            std::fs::create_dir_all(&thumb_dir)?;
            std::fs::create_dir_all(&output_root)?;
            // allow the frontend to load thumbnails over the asset protocol
            app.asset_protocol_scope().allow_directory(&thumb_dir, true)?;
            app.manage(AppState {
                files: Mutex::new(HashMap::new()),
                outputs: Arc::new(Mutex::new(HashMap::new())),
                batch_running: Mutex::new(false),
                cancel: Mutex::new(None),
                thumb_dir,
                output_root,
                app_data,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::analyze_files,
            commands::get_templates,
            commands::save_custom_template,
            commands::delete_custom_template,
            commands::start_batch,
            commands::cancel_batch,
            commands::export_files,
            commands::pick_folder,
            commands::get_app_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
