use std::path::PathBuf;
use tauri::{AppHandle, State};

use crate::services::scraper_service;
use crate::state::AppState;

#[tauri::command]
pub async fn run_scraper(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let (album_dir, input_dir) = {
        let config = state.0.lock().map_err(|e| e.to_string())?;
        (
            PathBuf::from(&config.album_dir),
            PathBuf::from(&config.album_input_dir),
        )
    };

    scraper_service::run(&app, &album_dir, &input_dir)
        .await
        .map_err(|e| e.to_string())
}
