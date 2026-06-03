use std::fs;
use tauri::{AppHandle, Manager};

use crate::core::errors::AppError;
use crate::core::state::AppConfig;

fn config_path(app: &AppHandle) -> Result<std::path::PathBuf, AppError> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|_| AppError::Io("Failed to resolve config directory".into()))?;
    Ok(dir.join("config.json"))
}

fn detect_bdo_dir(app: &AppHandle) -> Option<String> {
    let docs = app.path().document_dir().ok()?;
    let candidate = docs.join("Black Desert");
    if candidate.is_dir() {
        return Some(candidate.to_string_lossy().into_owned());
    }
    None
}

pub fn load(app: &AppHandle) -> AppConfig {
    let Ok(path) = config_path(app) else {
        return AppConfig::default();
    };
    let Ok(content) = fs::read_to_string(&path) else {
        let mut config = AppConfig::default();
        if let Some(dir) = detect_bdo_dir(app) {
            config.bdo_docs_dir = dir;
            let _ = save(app, &config);
        }
        return config;
    };
    let mut config: AppConfig = serde_json::from_str(&content).unwrap_or_default();
    if config.bdo_docs_dir.is_empty() {
        if let Some(dir) = detect_bdo_dir(app) {
            config.bdo_docs_dir = dir;
            let _ = save(app, &config);
        }
    }
    config
}

pub fn save(app: &AppHandle, config: &AppConfig) -> Result<(), AppError> {
    let path = config_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&path, content)?;
    if !config.bdo_docs_dir.is_empty() {
        let _ = fs::create_dir_all(config.db_dir());
        let _ = fs::create_dir_all(config.presets_dir());
        let _ = fs::create_dir_all(config.to_download_dir());
        let _ = fs::create_dir_all(config.customization_dir());
    }
    Ok(())
}
