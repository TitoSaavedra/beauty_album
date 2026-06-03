use tauri::State;

use crate::beauty::service;
use crate::core::logger::Logger;
use crate::core::state::{AppState, DbConn};
use crate::db::repositories::class_repo::ClassRepository;
use crate::db::repositories::preset_repo::PresetRepository;

#[tauri::command]
pub fn is_db_ready(db: State<'_, DbConn>) -> bool {
    db.0.get().is_some()
}

#[tauri::command]
pub async fn get_classes(db: State<'_, DbConn>) -> Result<Vec<serde_json::Value>, String> {
    let conn = db.0.get().ok_or("Database not initialized")?;
    service::get_all_classes(conn).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_presets(
    class_name: String,
    #[allow(non_snake_case)] sortBy: Option<String>,
    search: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
    state: State<'_, AppState>,
    db: State<'_, DbConn>,
) -> Result<Vec<serde_json::Value>, String> {
    let conn = db.0.get().ok_or("Database not initialized")?;
    let (presets_dir, popular_dir) = {
        let config = state.0.lock().map_err(|e| e.to_string())?;
        (config.presets_dir(), config.popular_dir())
    };
    service::get_presets(
        conn,
        &presets_dir,
        &popular_dir,
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
pub fn inject_preset(download_path: String, state: State<AppState>, db: State<DbConn>) -> Result<(), String> {
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

    if let Some(conn) = db.0.get() {
        let conn = conn.clone();
        let fname = file_name.to_string();
        tauri::async_runtime::spawn(async move {
            Logger::new(&conn, "album").tag("USER", &format!("Injected preset: {}", fname)).await;
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
pub async fn get_popular_presets(
    class_name: String,
    #[allow(non_snake_case)] sortBy: Option<String>,
    search: Option<String>,
    #[allow(non_snake_case)] sinceTs: Option<i64>,
    offset: Option<i64>,
    limit: Option<i64>,
    region: Option<String>,
    state: State<'_, AppState>,
    db: State<'_, DbConn>,
) -> Result<serde_json::Value, String> {
    let conn = db.0.get().ok_or("Database not initialized")?;
    let (popular_dir, presets_dir) = {
        let config = state.0.lock().map_err(|e| e.to_string())?;
        (config.popular_dir(), config.presets_dir())
    };

    let presets = service::get_popular_presets(
        conn,
        &popular_dir,
        &presets_dir,
        &class_name,
        sortBy.as_deref().unwrap_or("downloads"),
        search.as_deref().unwrap_or(""),
        sinceTs.unwrap_or(0),
        offset.unwrap_or(0),
        limit.unwrap_or(50),
        vec![],
        region.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| e.to_string())?;

    let wanted = service::get_popular_presets(
        conn,
        &popular_dir,
        &presets_dir,
        &class_name,
        "downloads",
        "",
        sinceTs.unwrap_or(0),
        0,
        1000,
        vec![],
        region.as_deref().unwrap_or(""),
    )
    .await
    .map_err(|e| e.to_string())?
    .into_iter()
    .filter(|p| p.get("is_wanted").and_then(|v| v.as_bool()).unwrap_or(false))
    .collect::<Vec<_>>();

    Ok(serde_json::json!({
        "presets": presets,
        "wanted": wanted,
    }))
}

#[tauri::command]
pub async fn get_popular_stats(
    class_name: String,
    db: State<'_, DbConn>,
) -> Result<serde_json::Value, String> {
    let conn = db.0.get().ok_or("Database not initialized")?;
    service::get_popular_stats(conn, &class_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_popular_regions(db: State<'_, DbConn>) -> Result<Vec<String>, String> {
    let conn = db.0.get().ok_or("Database not initialized")?;
    PresetRepository::get_regions(conn).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_wanted(db: State<'_, DbConn>) -> Result<Vec<String>, String> {
    let conn = db.0.get().ok_or("Database not initialized")?;
    let ids = PresetRepository::wanted_get_ids(conn).await.map_err(|e| e.to_string())?;
    Ok(ids.iter().map(|id| id.to_string()).collect())
}

#[tauri::command]
pub async fn set_wanted(ids: Vec<String>, db: State<'_, DbConn>) -> Result<(), String> {
    let conn = db.0.get().ok_or("Database not initialized")?;
    let parsed: Vec<i64> = ids.iter()
        .filter_map(|s| s.parse().ok())
        .collect();
    PresetRepository::wanted_set_bulk(conn, &parsed).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_wanted(preset_id: String, db: State<'_, DbConn>) -> Result<bool, String> {
    let conn = db.0.get().ok_or("Database not initialized")?;
    let id: i64 = preset_id.parse().map_err(|_| "Invalid preset id")?;
    let current = PresetRepository::wanted_get(conn, id).await.map_err(|e| e.to_string())?;
    let new_val = if current == 0 { 1i64 } else { 0i64 };
    PresetRepository::wanted_toggle(conn, id, new_val).await.map_err(|e| e.to_string())?;
    Ok(new_val == 1)
}

#[tauri::command]
pub async fn discard_preset(preset_id: String, db: State<'_, DbConn>) -> Result<(), String> {
    let conn = db.0.get().ok_or("Database not initialized")?;
    let id: i64 = preset_id.parse().map_err(|_| "Invalid preset id")?;
    PresetRepository::discard(conn, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_preset_by_id(
    preset_id: String,
    state: State<'_, AppState>,
    db: State<'_, DbConn>,
) -> Result<serde_json::Value, String> {
    let conn = db.0.get().ok_or("Database not initialized")?;
    let id: i64 = preset_id.parse().map_err(|_| "Invalid preset id")?;
    let (popular_dir, presets_dir) = {
        let config = state.0.lock().map_err(|e| e.to_string())?;
        (config.popular_dir(), config.presets_dir())
    };
    PresetRepository::get_preset_by_id(conn, &popular_dir, &presets_dir, id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Preset not found".to_string())
}

#[tauri::command]
pub async fn get_class_favorites(db: State<'_, DbConn>) -> Result<Vec<String>, String> {
    let conn = db.0.get().ok_or("Database not initialized")?;
    ClassRepository::get_favorites(conn).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_class_favorite(
    class_name: String,
    is_favorite: bool,
    db: State<'_, DbConn>,
) -> Result<(), String> {
    let conn = db.0.get().ok_or("Database not initialized")?;
    ClassRepository::set_favorite(conn, &class_name, is_favorite)
        .await
        .map_err(|e| e.to_string())
}

