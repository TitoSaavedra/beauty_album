use std::path::Path;

use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tauri::AppHandle;

use crate::core::errors::AppError;
use crate::core::events;

pub async fn open(db_path: &Path, app: &AppHandle) -> Result<SqlitePool, AppError> {
    events::emit_init_progress(app, "Opening database...");

    if db_path.exists() {
        let bak = db_path.parent().unwrap_or(Path::new(".")).join(
            format!("{}.bak", db_path.file_name().unwrap_or_default().to_string_lossy()),
        );
        let _ = std::fs::copy(db_path, &bak);
    }

    let pool = SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true),
    )
    .await?;

    sqlx::query("PRAGMA journal_mode=WAL").execute(&pool).await?;
    sqlx::query("PRAGMA foreign_keys=ON").execute(&pool).await?;

    events::emit_init_progress(app, "Running migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;

    events::emit_init_progress(app, "Database ready");
    Ok(pool)
}
