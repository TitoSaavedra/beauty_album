use tauri::State;

use crate::services::{log::Logger, presets::album_service};
use crate::state::{AppState, DbPool};

#[tauri::command]
pub fn is_db_ready(db: State<'_, DbPool>) -> bool {
    db.0.get().is_some()
}

#[tauri::command]
pub async fn get_classes(db: State<'_, DbPool>) -> Result<Vec<serde_json::Value>, String> {
    let pool = db.0.get().ok_or("Database not initialized")?;
    album_service::get_classes_for_presets(pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_presets(
    class_name: String,
    #[allow(non_snake_case)] sortBy: Option<String>,
    search: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
    state: State<'_, AppState>,
    db: State<'_, DbPool>,
) -> Result<Vec<serde_json::Value>, String> {
    let pool = db.0.get().ok_or("Database not initialized")?;
    let presets_dir = state.0.lock().map_err(|e| e.to_string())?.presets_dir();
    album_service::get_presets(
        pool,
        &presets_dir,
        &class_name,
        sortBy.as_deref().unwrap_or("downloads"),
        search.as_deref().unwrap_or(""),
        offset.unwrap_or(0),
        limit.unwrap_or(50),
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn inject_preset(download_path: String, state: State<AppState>, db: State<DbPool>) -> Result<(), String> {
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

    if let Some(pool) = db.0.get() {
        let pool = pool.clone();
        let fname = file_name.to_string();
        tauri::async_runtime::spawn(async move {
            Logger::new(&pool, "album").tag("USER", &format!("Injected preset: {}", fname)).await;
        });
    }

    Ok(())
}

#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err(format!("File not found: {}", path));
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("cmd")
            .creation_flags(0x08000000)
            .args(["/c", "start", "", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(&path).spawn().map_err(|e| e.to_string())?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(&path).spawn().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("cmd")
            .creation_flags(0x08000000)
            .args(["/c", "start", "", &url])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn open_logs(db: State<DbPool>) -> Result<(), String> {
    if let Some(pool) = db.0.get() {
        let pool = pool.clone();
        tauri::async_runtime::spawn(async move {
            Logger::new(&pool, "album").tag("USER", "Opened log viewer").await;
        });
    }
    Ok(())
}
