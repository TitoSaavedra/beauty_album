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

    scrapper_service::run(&app, &album_dir, cancel.0.clone())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_scrapper(cancel: State<'_, ScrapperCancelToken>) {
    cancel.0.store(true, Ordering::Relaxed);
}
