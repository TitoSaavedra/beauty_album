use tauri::State;

use crate::core::state::DbConn;
use crate::db::repositories::log_repo::LogRepository;

#[tauri::command]
pub async fn get_logs(
    tag: Option<String>,
    source: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
    db: State<'_, DbConn>,
) -> Result<Vec<serde_json::Value>, String> {
    let conn = db.0.get().ok_or("Database not initialized")?;
    LogRepository::get(
        conn,
        tag.as_deref(),
        source.as_deref(),
        limit.unwrap_or(100),
        offset.unwrap_or(0),
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_log_stats(db: State<'_, DbConn>) -> Result<serde_json::Value, String> {
    let conn = db.0.get().ok_or("Database not initialized")?;

    let total = LogRepository::stats_total(conn).await.map_err(|e| e.to_string())?;
    let errors = LogRepository::stats_errors(conn).await.map_err(|e| e.to_string())?;
    let by_source = LogRepository::stats_by_source(conn).await.map_err(|e| e.to_string())?;
    let by_tag = LogRepository::stats_by_tag(conn).await.map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "total":     total,
        "errors":    errors,
        "by_source": by_source,
        "by_tag":    by_tag,
    }))
}
