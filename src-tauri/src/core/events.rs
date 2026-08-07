use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::core::state::AppConfig;

#[derive(Serialize, Clone, Debug)]
pub enum ProgressType {
    #[serde(rename = "preset")]
    Preset,
    #[serde(rename = "popular")]
    Popular,
}

#[derive(Serialize, Clone, Debug)]
pub enum ProgressStatus {
    #[serde(rename = "processing")]
    Processing,
    #[serde(rename = "metadata")]
    Metadata,
    #[serde(rename = "done")]
    Done,
}

#[derive(Serialize, Clone)]
pub struct ScrapperProgress {
    pub preset_id: String,
    pub status: ProgressStatus,
    pub message: String,
    pub class_name: String,
    pub class_id: u32,
    pub current: usize,
    pub total: usize,
    pub progress_type: ProgressType,
}

pub fn emit_progress(app: &AppHandle, progress: ScrapperProgress) {
    let _ = app.emit("scrapper_progress", progress);
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

pub fn emit_sync_loading(app: &AppHandle, message: &str) {
    let _ = app.emit("sync_loading", message);
}
