use std::fs;
use tauri::{AppHandle, Manager};

use crate::errors::AppError;
use crate::state::AppConfig;

fn config_path(app: &AppHandle) -> Result<std::path::PathBuf, AppError> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|_| AppError::Io("Failed to resolve config directory".into()))?;
    Ok(dir.join("config.json"))
}

pub fn load(app: &AppHandle) -> AppConfig {
    let Ok(path) = config_path(app) else {
        return AppConfig::default();
    };
    let Ok(content) = fs::read_to_string(&path) else {
        return AppConfig::default();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn save(app: &AppHandle, config: &AppConfig) -> Result<(), AppError> {
    let path = config_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&path, content)?;
    if !config.bdo_docs_dir.is_empty() {
        let _ = fs::create_dir_all(config.presets_dir());
        let _ = fs::create_dir_all(config.customization_dir());
        let _ = fs::create_dir_all(config.to_download_dir());
        let _ = fs::create_dir_all(config.logs_dir());
    }
    Ok(())
}
