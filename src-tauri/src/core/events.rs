use serde::Serialize;
use tauri::{AppHandle, Emitter};

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

pub fn emit_scrapper_done(app: &AppHandle, msg: &str) {
    let _ = app.emit("scrapper_done", msg);
}

pub fn emit_refresh_album(app: &AppHandle) {
    let _ = app.emit("refresh_album", ());
}

pub fn emit_db_ready(app: &AppHandle, ok: bool) {
    let _ = app.emit("db_ready", ok);
}

pub fn emit_init_progress(app: &AppHandle, message: &str) {
    let _ = app.emit("init_progress", message);
}

pub fn emit_class_count_updated(app: &AppHandle, class_id: u32, count: i64, is_popular: bool) {
    let _ = app.emit("class_count_updated", serde_json::json!({
        "class_id": class_id,
        "count": count,
        "is_popular": is_popular
    }));
}
