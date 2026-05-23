use sqlx::Row;
use tauri::State;

use crate::state::DbPool;

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

    let rows = match (tag.as_deref(), source.as_deref()) {
        (Some(t), Some(s)) => sqlx::query(
            "SELECT id, ts, tag, source, msg FROM logs WHERE tag=? AND source=? ORDER BY ts DESC LIMIT ? OFFSET ?",
        )
        .bind(t).bind(s).bind(limit).bind(offset)
        .fetch_all(pool).await,
        (Some(t), None) => sqlx::query(
            "SELECT id, ts, tag, source, msg FROM logs WHERE tag=? ORDER BY ts DESC LIMIT ? OFFSET ?",
        )
        .bind(t).bind(limit).bind(offset)
        .fetch_all(pool).await,
        (None, Some(s)) => sqlx::query(
            "SELECT id, ts, tag, source, msg FROM logs WHERE source=? ORDER BY ts DESC LIMIT ? OFFSET ?",
        )
        .bind(s).bind(limit).bind(offset)
        .fetch_all(pool).await,
        (None, None) => sqlx::query(
            "SELECT id, ts, tag, source, msg FROM logs ORDER BY ts DESC LIMIT ? OFFSET ?",
        )
        .bind(limit).bind(offset)
        .fetch_all(pool).await,
    }
    .map_err(|e| e.to_string())?;

    Ok(rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id":     r.get::<i64, _>("id"),
                "ts":     r.get::<i64, _>("ts"),
                "tag":    r.get::<String, _>("tag"),
                "source": r.get::<String, _>("source"),
                "msg":    r.get::<String, _>("msg"),
            })
        })
        .collect())
}

#[tauri::command]
pub async fn get_log_stats(db: State<'_, DbPool>) -> Result<serde_json::Value, String> {
    let pool = db.0.get().ok_or("Database not initialized")?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM logs")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    let errors: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM logs WHERE tag='ERR'")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    let sources = sqlx::query(
        "SELECT source, COUNT(*) as n FROM logs GROUP BY source ORDER BY n DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let tags = sqlx::query(
        "SELECT tag, COUNT(*) as n FROM logs GROUP BY tag ORDER BY n DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

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
