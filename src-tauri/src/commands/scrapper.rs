use std::sync::atomic::Ordering;
use tauri::{AppHandle, State};

use crate::services::scrapper_service;
use crate::state::{AppState, ScrapperCancelToken};

#[tauri::command]
pub async fn run_scrapper(
    app: AppHandle,
    state: State<'_, AppState>,
    cancel: State<'_, ScrapperCancelToken>,
) -> Result<String, String> {
    cancel.0.store(false, Ordering::Relaxed);

    let logs_dir = {
        let config = state.0.lock().map_err(|e| e.to_string())?;
        config.logs_dir()
    };

    scrapper_service::write_log(&logs_dir, "[USER ] Sync requested").await;

    scrapper_service::run(&app, &logs_dir, cancel.0.clone())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_scrapper(cancel: State<'_, ScrapperCancelToken>, state: State<'_, AppState>) {
    cancel.0.store(true, Ordering::Relaxed);
    let logs_dir = state.0.lock().ok().map(|c| c.logs_dir());
    if let Some(dir) = logs_dir {
        scrapper_service::write_log_sync(&dir, "[USER ] Stop sync requested");
    }
}

#[tauri::command]
pub async fn check_pending(state: State<'_, AppState>) -> Result<usize, String> {
    let logs_dir = {
        let config = state.0.lock().map_err(|e| e.to_string())?;
        config.logs_dir()
    };
    let count = scrapper_service::pending_count()
        .await
        .map_err(|e| e.to_string())?;
    scrapper_service::write_log(&logs_dir, &format!("[INFO ] Pending check: {} preset(s) need sync", count)).await;
    Ok(count)
}
