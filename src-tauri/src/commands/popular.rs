use std::sync::atomic::Ordering;

use tauri::{AppHandle, State};

use crate::services::{album_service, popular_service};
use crate::state::{AppState, ScrapperCancelToken};

#[tauri::command]
pub async fn sync_popular(
    app: AppHandle,
    state: State<'_, AppState>,
    cancel: State<'_, ScrapperCancelToken>,
) -> Result<String, String> {
    cancel.0.store(false, Ordering::Relaxed);

    let (popular_dir, presets_dir, classes_dir, logs_dir) = {
        let config = state.0.lock().map_err(|e| e.to_string())?;
        (config.popular_dir(), config.presets_dir(), config.classes_dir(), config.logs_dir())
    };

    popular_service::sync_popular(
        &app,
        &popular_dir,
        &presets_dir,
        &classes_dir,
        &logs_dir,
        cancel.0.clone(),
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_popular_classes(state: State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    let config = state.0.lock().map_err(|e| e.to_string())?;
    album_service::get_classes(&config.popular_dir(), &config.classes_dir()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_popular_presets(class_name: String, state: State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    let config = state.0.lock().map_err(|e| e.to_string())?;
    let popular_dir = config.popular_dir();
    let presets_dir = config.presets_dir();

    let mut presets = album_service::get_presets(&popular_dir, &class_name).map_err(|e| e.to_string())?;

    presets.retain(|p| {
        let id = p["preset_id"].as_str().unwrap_or("");
        !popular_dir.join(&class_name).join(id).join(".discard").exists()
    });

    for preset in &mut presets {
        let id = preset["preset_id"].as_str().unwrap_or("").to_string();
        let is_downloaded = std::fs::read_dir(&presets_dir)
            .map(|rd| rd.flatten().any(|e| e.path().join(&id).join(".ok").exists()))
            .unwrap_or(false);
        preset["is_downloaded"] = serde_json::json!(is_downloaded);
        preset["is_popular"] = serde_json::json!(true);
    }

    Ok(presets)
}

#[tauri::command]
pub fn get_wanted(state: State<AppState>) -> Result<Vec<String>, String> {
    let path = state.0.lock().map_err(|e| e.to_string())?.wanted_path();
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_wanted(ids: Vec<String>, state: State<AppState>) -> Result<(), String> {
    let path = state.0.lock().map_err(|e| e.to_string())?.wanted_path();
    let content = serde_json::to_string(&ids).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn discard_preset(preset_id: String, state: State<AppState>) -> Result<(), String> {
    let config = state.0.lock().map_err(|e| e.to_string())?;
    let popular_dir = config.popular_dir();

    let Ok(rd) = std::fs::read_dir(&popular_dir) else {
        return Err("Popular directory not found".to_string());
    };

    for class_entry in rd.flatten() {
        let candidate = class_entry.path().join(&preset_id);
        if candidate.is_dir() {
            std::fs::write(candidate.join(".discard"), b"").map_err(|e| e.to_string())?;
            return Ok(());
        }
    }

    Err(format!("Preset {} not found in popular dir", preset_id))
}
