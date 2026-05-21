use tauri::State;

use crate::services::{album_service, scrapper_service};
use crate::state::AppState;

#[tauri::command]
pub fn get_classes(use_test: bool, state: State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    let config = state.0.lock().map_err(|e| e.to_string())?;
    let dir = if use_test { config.test_dir() } else { config.presets_dir() };
    album_service::get_classes(&dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_presets(
    class_name: String,
    use_test: bool,
    state: State<AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let config = state.0.lock().map_err(|e| e.to_string())?;
    let dir = if use_test { config.test_dir() } else { config.presets_dir() };
    album_service::get_presets(&dir, &class_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn inject_preset(download_path: String, state: State<AppState>) -> Result<(), String> {
    use std::fs;
    use std::path::Path;

    let config = state.0.lock().map_err(|e| e.to_string())?;
    let out = config.customization_dir();

    if !out.is_dir() {
        return Err(format!("Output directory not found: {}", out.display()));
    }

    for entry in fs::read_dir(&out).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
        } else {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
    }

    let src = Path::new(&download_path);
    if !src.exists() {
        return Err(format!("Preset file not found: {}", download_path));
    }

    let file_name = src
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid file name".to_string())?;

    fs::copy(src, out.join(file_name)).map_err(|e| e.to_string())?;

    scrapper_service::write_log_sync(&config.logs_dir(), &format!("[USER ] Injected preset: {}", file_name));

    Ok(())
}

#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err(format!("File not found: {}", path));
    }

    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(["/c", "start", "", &path])
        .spawn()
        .map_err(|e| e.to_string())?;

    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn open_logs(state: State<AppState>) -> Result<(), String> {
    let config = state.0.lock().map_err(|e| e.to_string())?;
    let logs = config.logs_dir();
    scrapper_service::write_log_sync(&logs, "[USER ] Opened logs in VS Code");
    use std::os::windows::process::CommandExt;
    std::process::Command::new("cmd")
        .creation_flags(0x08000000)
        .arg("/c")
        .arg("code")
        .arg(logs.join("tauri.log"))
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
