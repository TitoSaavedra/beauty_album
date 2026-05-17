use tauri::{AppHandle, State};

use crate::services::config_service;
use crate::state::{AppConfig, AppState};

#[tauri::command]
pub fn get_config(state: State<AppState>) -> Result<AppConfig, String> {
    let config = state.0.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
pub fn save_config(
    config: AppConfig,
    app: AppHandle,
    state: State<AppState>,
) -> Result<(), String> {
    config_service::save(&app, &config).map_err(|e| e.to_string())?;
    *state.0.lock().map_err(|e| e.to_string())? = config;
    Ok(())
}
