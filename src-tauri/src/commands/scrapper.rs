use std::sync::atomic::Ordering;
use tauri::{AppHandle, State};

use crate::services::scrapper_service;
use crate::state::{AppState, ScrapperCancelToken};

#[tauri::command]
pub async fn run_scrapper(
    app: AppHandle,
    dev_mode: bool,
    state: State<'_, AppState>,
    cancel: State<'_, ScrapperCancelToken>,
) -> Result<String, String> {
    cancel.0.store(false, Ordering::Relaxed);

    let (input_dir, presets_dir, logs_dir) = {
        let config = state.0.lock().map_err(|e| e.to_string())?;
        let out = if dev_mode { config.test_dir() } else { config.presets_dir() };
        (config.to_download_dir(), out, config.logs_dir())
    };

    let log_prefix = if dev_mode { "[DEV  ]" } else { "[USER ]" };
    scrapper_service::write_log(&logs_dir, &format!("{} Sync requested", log_prefix)).await;

    scrapper_service::run(&app, &input_dir, &presets_dir, &logs_dir, cancel.0.clone(), dev_mode)
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
    let (input_dir, presets_dir, logs_dir) = {
        let config = state.0.lock().map_err(|e| e.to_string())?;
        (config.to_download_dir(), config.presets_dir(), config.logs_dir())
    };
    let count = scrapper_service::pending_count(&input_dir, &presets_dir);
    scrapper_service::write_log(&logs_dir, &format!("[INFO ] Pending check: {} preset(s) need sync", count)).await;
    Ok(count)
}
