use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::core::state::AppConfig;

#[derive(Serialize, Clone)]
pub struct ScrapperProgress {
    pub preset_id: String,
    pub status: String,
    pub message: String,
    pub class_name: String,
    pub class_id: u32,
    pub current: usize,
    pub total: usize,
}

pub fn emit_progress(app: &AppHandle, progress: ScrapperProgress) {
    let _ = app.emit("scrapper_progress", progress);
}

pub fn emit_folder_changed(app: &AppHandle, files: Vec<String>) {
    let _ = app.emit("folder_changed", files);
}

pub fn emit_popular_progress(app: &AppHandle, progress: ScrapperProgress) {
    let _ = app.emit("popular_progress", progress);
}

pub fn emit_db_ready(app: &AppHandle, ok: bool) {
    let _ = app.emit("db_ready", ok);
}

pub fn emit_init_progress(app: &AppHandle, message: &str) {
    let _ = app.emit("init_progress", message);
}


pub fn emit_config_loaded(app: &AppHandle, config: &AppConfig) {
    let _ = app.emit("config_loaded", config);
}
