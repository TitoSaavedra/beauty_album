use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::services::scrapper_service::ScrapperProgress;

#[derive(Serialize, Clone)]
pub struct ClassInitProgress {
    pub current: usize,
    pub total: usize,
    pub class_name: String,
}

pub fn emit_progress(app: &AppHandle, progress: ScrapperProgress) {
    let _ = app.emit("scrapper_progress", progress);
}

pub fn emit_class_init_progress(app: &AppHandle, progress: ClassInitProgress) {
    let _ = app.emit("class_init_progress", progress);
}

pub fn emit_folder_changed(app: &AppHandle, files: Vec<String>) {
    let _ = app.emit("folder_changed", files);
}

pub fn emit_refresh_album(app: &AppHandle) {
    let _ = app.emit("refresh_album", ());
}
