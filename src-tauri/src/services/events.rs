use tauri::{AppHandle, Emitter};

use crate::services::scrapper_service::ScrapperProgress;

pub fn emit_progress(app: &AppHandle, progress: ScrapperProgress) {
    let _ = app.emit("scrapper_progress", progress);
}

pub fn emit_folder_changed(app: &AppHandle, files: Vec<String>) {
    let _ = app.emit("folder_changed", files);
}
