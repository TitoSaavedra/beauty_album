use std::path::PathBuf;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, State};

use crate::services::scrapper_service;
use crate::state::{AppState, ScrapperCancelToken};

#[tauri::command]
pub async fn run_scrapper(
    app: AppHandle,
    state: State<'_, AppState>,
    cancel: State<'_, ScrapperCancelToken>,
) -> Result<(), String> {
    cancel.0.store(false, Ordering::Relaxed);

    let album_dir = {
        let config = state.0.lock().map_err(|e| e.to_string())?;
        PathBuf::from(&config.album_dir)
    };

    scrapper_service::write_log(&album_dir, "[USER ] Sync requested").await;

    scrapper_service::run(&app, &album_dir, cancel.0.clone())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_scrapper(cancel: State<'_, ScrapperCancelToken>, state: State<'_, AppState>) {
    cancel.0.store(true, Ordering::Relaxed);
    let album_dir = state.0.lock().ok().map(|c| PathBuf::from(&c.album_dir));
    if let Some(dir) = album_dir {
        scrapper_service::write_log_sync(&dir, "[USER ] Stop sync requested");
    }
}

#[tauri::command]
pub async fn check_pending(state: State<'_, AppState>) -> Result<usize, String> {
    let album_dir = {
        let config = state.0.lock().map_err(|e| e.to_string())?;
        PathBuf::from(&config.album_dir)
    };
    let count = scrapper_service::pending_count()
        .await
        .map_err(|e| e.to_string())?;
    scrapper_service::write_log(&album_dir, &format!("[INFO ] Pending check: {} preset(s) need sync", count)).await;
    Ok(count)
}
