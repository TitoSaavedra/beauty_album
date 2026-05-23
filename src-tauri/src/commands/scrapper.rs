use std::sync::atomic::Ordering;
use tauri::{AppHandle, State};

use crate::services::{log::Logger, scraping::scrapper_service};
use crate::state::{AppState, DbPool, ScrapperCancelToken};

#[tauri::command]
pub async fn run_scrapper(
    app: AppHandle,
    state: State<'_, AppState>,
    db: State<'_, DbPool>,
    cancel: State<'_, ScrapperCancelToken>,
) -> Result<String, String> {
    cancel.0.store(false, Ordering::Relaxed);
    let pool = db.0.get().ok_or("Database not initialized")?;
    let (input_dir, presets_dir, popular_dir) = {
        let config = state.0.lock().map_err(|e| e.to_string())?;
        (config.to_download_dir(), config.presets_dir(), config.popular_dir())
    };
    Logger::new(pool, "scrapper").tag("USER", "Sync requested").await;
    scrapper_service::run(&app, pool, &input_dir, &presets_dir, &popular_dir, cancel.0.clone())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_scrapper(
    cancel: State<'_, ScrapperCancelToken>,
    db: State<'_, DbPool>,
) -> Result<(), String> {
    cancel.0.store(true, Ordering::Relaxed);
    if let Some(pool) = db.0.get() {
        Logger::new(pool, "scrapper").tag("USER", "Stop sync requested").await;
    }
    Ok(())
}

#[tauri::command]
pub async fn check_pending(
    state: State<'_, AppState>,
    db: State<'_, DbPool>,
) -> Result<usize, String> {
    let pool = db.0.get().ok_or("Database not initialized")?;
    let input_dir = state.0.lock().map_err(|e| e.to_string())?.to_download_dir();
    let count = scrapper_service::pending_count(pool, &input_dir).await;
    Logger::new(pool, "scrapper").tag("INFO", &format!("Pending check: {} preset(s) need sync", count)).await;
    Ok(count)
}

#[tauri::command]
pub async fn check_pending_image2(
    state: State<'_, AppState>,
    db: State<'_, DbPool>,
) -> Result<usize, String> {
    use sqlx::Row;
    let pool = db.0.get().ok_or("Database not initialized")?;
    let presets_dir = state.0.lock().map_err(|e| e.to_string())?.presets_dir();

    let rows = sqlx::query(
        "SELECT p.id, c.display, p.image_2 FROM presets p
         JOIN classes c ON c.id_garmoth = p.class_id
         WHERE p.image_2 IS NOT NULL AND p.is_ok = 1 AND p.is_popular = 0",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let count = rows
        .iter()
        .filter(|row| {
            let id: i64 = row.get(0);
            let display: String = row.get(1);
            let image_2: String = row.get(2);
            !presets_dir
                .join(&display)
                .join(id.to_string())
                .join(&image_2)
                .exists()
        })
        .count();

    if count > 0 {
        Logger::new(pool, "scrapper")
            .tag("INFO", &format!("Pending image_2: {} preset(s)", count))
            .await;
    }
    Ok(count)
}
