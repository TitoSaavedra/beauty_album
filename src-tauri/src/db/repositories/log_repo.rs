use sea_orm::{ConnectionTrait, DbBackend, DbErr, Statement, Value};
use serde_json;

pub struct LogRepository;

impl LogRepository {
    pub async fn insert(
        db: &impl ConnectionTrait,
        ts: i64,
        tag: &str,
        source: &str,
        msg: &str,
    ) -> Result<i64, DbErr> {
        let result = db
            .execute(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "INSERT INTO logs (ts, tag, source, msg) VALUES (?, ?, ?, ?)",
                [ts.into(), tag.into(), source.into(), msg.into()],
            ))
            .await?;
        Ok(result.last_insert_id() as i64)
    }

    pub async fn get(
        db: &impl ConnectionTrait,
        tag: Option<&str>,
        source: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<serde_json::Value>, DbErr> {
        let rows = db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "SELECT id, ts, tag, source, msg FROM logs
                 WHERE (? IS NULL OR tag = ?)
                   AND (? IS NULL OR source = ?)
                 ORDER BY ts DESC
                 LIMIT ? OFFSET ?",
                [
                    tag.map(Value::from).unwrap_or(Value::Bool(None)),
                    tag.map(Value::from).unwrap_or(Value::Bool(None)),
                    source.map(Value::from).unwrap_or(Value::Bool(None)),
                    source.map(Value::from).unwrap_or(Value::Bool(None)),
                    limit.into(),
                    offset.into(),
                ],
            ))
            .await?;

        Ok(rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id":     r.try_get::<i64>("", "id").unwrap_or(0),
                    "ts":     r.try_get::<i64>("", "ts").unwrap_or(0),
                    "tag":    r.try_get::<String>("", "tag").unwrap_or_default(),
                    "source": r.try_get::<String>("", "source").unwrap_or_default(),
                    "msg":    r.try_get::<String>("", "msg").unwrap_or_default(),
                })
            })
            .collect())
    }

    pub async fn stats_total(db: &impl ConnectionTrait) -> Result<i64, DbErr> {
        let row = db
            .query_one(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT COUNT(*) AS n FROM logs".to_string(),
            ))
            .await?;
        Ok(row.map(|r| r.try_get::<i64>("", "n").unwrap_or(0)).unwrap_or(0))
    }

    pub async fn stats_errors(db: &impl ConnectionTrait) -> Result<i64, DbErr> {
        let row = db
            .query_one(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT COUNT(*) AS n FROM logs WHERE tag = 'ERR'".to_string(),
            ))
            .await?;
        Ok(row.map(|r| r.try_get::<i64>("", "n").unwrap_or(0)).unwrap_or(0))
    }

    pub async fn stats_by_source(db: &impl ConnectionTrait) -> Result<Vec<serde_json::Value>, DbErr> {
        let rows = db
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT source, COUNT(*) AS n FROM logs GROUP BY source ORDER BY n DESC".to_string(),
            ))
            .await?;
        Ok(rows
            .iter()
            .map(|r| serde_json::json!({
                "source": r.try_get::<String>("", "source").unwrap_or_default(),
                "count":  r.try_get::<i64>("", "n").unwrap_or(0),
            }))
            .collect())
    }

    pub async fn stats_by_tag(db: &impl ConnectionTrait) -> Result<Vec<serde_json::Value>, DbErr> {
        let rows = db
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT tag, COUNT(*) AS n FROM logs GROUP BY tag ORDER BY n DESC".to_string(),
            ))
            .await?;
        Ok(rows
            .iter()
            .map(|r| serde_json::json!({
                "tag":   r.try_get::<String>("", "tag").unwrap_or_default(),
                "count": r.try_get::<i64>("", "n").unwrap_or(0),
            }))
            .collect())
    }
}
