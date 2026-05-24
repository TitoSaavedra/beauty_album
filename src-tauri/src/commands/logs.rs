use sqlx::Row;
use tauri::State;

use crate::state::DbPool;

const LOGS_GET: &str = include_str!("SQL/logs_get.sql");
const LOGS_STATS_TOTAL: &str = include_str!("SQL/logs_stats_total.sql");
const LOGS_STATS_ERRORS: &str = include_str!("SQL/logs_stats_errors.sql");
const LOGS_STATS_BY_SOURCE: &str = include_str!("SQL/logs_stats_by_source.sql");
const LOGS_STATS_BY_TAG: &str = include_str!("SQL/logs_stats_by_tag.sql");

#[tauri::command]
pub async fn get_logs(
    tag: Option<String>,
    source: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
    db: State<'_, DbPool>,
) -> Result<Vec<serde_json::Value>, String> {
    let pool = db.0.get().ok_or("Database not initialized")?;
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);

    let rows = sqlx::query(LOGS_GET)
        .bind(&tag).bind(&tag)
        .bind(&source).bind(&source)
        .bind(limit).bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(rows
        .iter()
        .map(|r| serde_json::json!({
            "id":     r.get::<i64, _>("id"),
            "ts":     r.get::<i64, _>("ts"),
            "tag":    r.get::<String, _>("tag"),
            "source": r.get::<String, _>("source"),
            "msg":    r.get::<String, _>("msg"),
        }))
        .collect())
}

#[tauri::command]
pub async fn get_log_stats(db: State<'_, DbPool>) -> Result<serde_json::Value, String> {
    let pool = db.0.get().ok_or("Database not initialized")?;

    let total: i64 = sqlx::query_scalar(LOGS_STATS_TOTAL)
        .fetch_one(pool).await.map_err(|e| e.to_string())?;

    let errors: i64 = sqlx::query_scalar(LOGS_STATS_ERRORS)
        .fetch_one(pool).await.map_err(|e| e.to_string())?;

    let sources = sqlx::query(LOGS_STATS_BY_SOURCE)
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

    let tags = sqlx::query(LOGS_STATS_BY_TAG)
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "total": total,
        "errors": errors,
        "by_source": sources.iter().map(|r| serde_json::json!({
            "source": r.get::<String, _>("source"),
            "count":  r.get::<i64, _>("n"),
        })).collect::<Vec<_>>(),
        "by_tag": tags.iter().map(|r| serde_json::json!({
            "tag":   r.get::<String, _>("tag"),
            "count": r.get::<i64, _>("n"),
        })).collect::<Vec<_>>(),
    }))
}
