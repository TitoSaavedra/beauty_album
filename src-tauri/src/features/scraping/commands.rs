use std::sync::atomic::Ordering;
use tauri::{AppHandle, State};

use crate::core::logger::Logger;
use crate::features::scraping::service;
use crate::core::state::{AppState, DbConn, ScrapperCancelToken};

#[tauri::command]
pub async fn run_scrapper(
    app: AppHandle,
    state: State<'_, AppState>,
    db: State<'_, DbConn>,
    cancel: State<'_, ScrapperCancelToken>,
) -> Result<String, String> {
    cancel.0.store(false, Ordering::Relaxed);
    let conn = db.0.get().ok_or("Database not initialized")?;
    let (input_dir, presets_dir, popular_dir) = {
        let config = state.0.lock().map_err(|e| e.to_string())?;
        (config.to_download_dir(), config.presets_dir(), config.popular_dir())
    };
    Logger::new(conn, "scrapper").tag("USER", "Sync requested").await;
    service::run(&app, conn, &input_dir, &presets_dir, &popular_dir, cancel.0.clone())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_scrapper(
    cancel: State<'_, ScrapperCancelToken>,
    db: State<'_, DbConn>,
) -> Result<(), String> {
    cancel.0.store(true, Ordering::Relaxed);
    if let Some(conn) = db.0.get() {
        Logger::new(conn, "scrapper").tag("USER", "Stop sync requested").await;
    }
    Ok(())
}

#[tauri::command]
pub async fn check_pending(
    state: State<'_, AppState>,
    db: State<'_, DbConn>,
) -> Result<usize, String> {
    let conn = db.0.get().ok_or("Database not initialized")?;
    let input_dir = state.0.lock().map_err(|e| e.to_string())?.to_download_dir();
    let count = service::pending_count(conn, &input_dir).await;
    Logger::new(conn, "scrapper").tag("INFO", &format!("Pending check: {} preset(s) need sync", count)).await;
    Ok(count)
}
