use std::sync::atomic::Ordering;

use sqlx::Row;
use tauri::{AppHandle, State};

use crate::services::presets::{album_service, popular_service};
use crate::state::{AppState, DbPool, ScrapperCancelToken};

const POPULAR_GET_WANTED: &str     = include_str!("../services/presets/SQL/popular_get_wanted.sql");
const POPULAR_WANTED_CLEAR: &str   = include_str!("../services/presets/SQL/popular_wanted_clear.sql");
const POPULAR_WANTED_SET: &str     = include_str!("../services/presets/SQL/popular_wanted_set.sql");
const POPULAR_WANTED_GET: &str     = include_str!("../services/presets/SQL/popular_wanted_get.sql");
const POPULAR_WANTED_TOGGLE: &str  = include_str!("../services/presets/SQL/popular_wanted_toggle.sql");
const POPULAR_DISCARD: &str        = include_str!("../services/presets/SQL/popular_discard.sql");
const CLASSES_GET_FAVORITES: &str  = include_str!("../services/presets/SQL/classes_get_favorites.sql");
const CLASSES_SET_FAVORITE: &str   = include_str!("../services/presets/SQL/classes_set_favorite.sql");

#[tauri::command]
pub async fn sync_popular(
    app: AppHandle,
    state: State<'_, AppState>,
    db: State<'_, DbPool>,
    cancel: State<'_, ScrapperCancelToken>,
) -> Result<String, String> {
    cancel.0.store(false, Ordering::Relaxed);

    let pool = db.0.get().ok_or("Database not initialized")?;
    let popular_dir = state.0.lock().map_err(|e| e.to_string())?.popular_dir();

    popular_service::sync_popular(&app, pool, &popular_dir, cancel.0.clone())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_popular_classes(
    #[allow(non_snake_case)] sinceTs: Option<i64>,
    db: State<'_, DbPool>,
) -> Result<Vec<serde_json::Value>, String> {
    let pool = db.0.get().ok_or("Database not initialized")?;
    album_service::get_classes_for_popular(pool, sinceTs.unwrap_or(0))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_popular_presets(
    class_name: String,
    #[allow(non_snake_case)] sortBy: Option<String>,
    search: Option<String>,
    #[allow(non_snake_case)] sinceTs: Option<i64>,
    offset: Option<i64>,
    limit: Option<i64>,
    state: State<'_, AppState>,
    db: State<'_, DbPool>,
) -> Result<Vec<serde_json::Value>, String> {
    let pool = db.0.get().ok_or("Database not initialized")?;
    let (popular_dir, presets_dir) = {
        let config = state.0.lock().map_err(|e| e.to_string())?;
        (config.popular_dir(), config.presets_dir())
    };

    album_service::get_popular_presets(
        pool,
        &popular_dir,
        &presets_dir,
        &class_name,
        sortBy.as_deref().unwrap_or("downloads"),
        search.as_deref().unwrap_or(""),
        sinceTs.unwrap_or(0),
        offset.unwrap_or(0),
        limit.unwrap_or(50),
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_popular_stats(
    class_name: String,
    db: State<'_, DbPool>,
) -> Result<serde_json::Value, String> {
    let pool = db.0.get().ok_or("Database not initialized")?;
    album_service::get_popular_stats(pool, &class_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_wanted(db: State<'_, DbPool>) -> Result<Vec<String>, String> {
    let pool = db.0.get().ok_or("Database not initialized")?;
    let rows = sqlx::query(POPULAR_GET_WANTED)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(rows.iter().map(|r| r.get::<i64, _>(0).to_string()).collect())
}

#[tauri::command]
pub async fn set_wanted(ids: Vec<String>, db: State<'_, DbPool>) -> Result<(), String> {
    let pool = db.0.get().ok_or("Database not initialized")?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    sqlx::query(POPULAR_WANTED_CLEAR)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    for id_str in &ids {
        if let Ok(id) = id_str.parse::<i64>() {
            sqlx::query(POPULAR_WANTED_SET)
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    tx.commit().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_wanted(preset_id: String, db: State<'_, DbPool>) -> Result<bool, String> {
    let pool = db.0.get().ok_or("Database not initialized")?;
    let id: i64 = preset_id.parse().map_err(|_| "Invalid preset id")?;
    let row = sqlx::query(POPULAR_WANTED_GET)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
    let current: i64 = row.map(|r| r.get(0)).unwrap_or(0);
    let new_val = if current == 0 { 1i64 } else { 0i64 };
    sqlx::query(POPULAR_WANTED_TOGGLE)
        .bind(new_val)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(new_val == 1)
}

#[tauri::command]
pub async fn discard_preset(preset_id: String, db: State<'_, DbPool>) -> Result<(), String> {
    let pool = db.0.get().ok_or("Database not initialized")?;
    let id: i64 = preset_id.parse().map_err(|_| "Invalid preset id")?;
    sqlx::query(POPULAR_DISCARD)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_class_favorites(db: State<'_, DbPool>) -> Result<Vec<String>, String> {
    let pool = db.0.get().ok_or("Database not initialized")?;
    let rows = sqlx::query(CLASSES_GET_FAVORITES)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(rows.iter().map(|r| r.get::<String, _>(0)).collect())
}

#[tauri::command]
pub async fn set_class_favorite(
    class_name: String,
    is_favorite: bool,
    db: State<'_, DbPool>,
) -> Result<(), String> {
    let pool = db.0.get().ok_or("Database not initialized")?;
    sqlx::query(CLASSES_SET_FAVORITE)
        .bind(is_favorite as i64)
        .bind(&class_name)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
