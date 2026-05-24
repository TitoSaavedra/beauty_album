use std::sync::OnceLock;
use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter};

static APP: OnceLock<AppHandle> = OnceLock::new();

pub fn init(app: &AppHandle) {
    let _ = APP.set(app.clone());
}

pub struct Logger {
    pool: SqlitePool,
    source: String,
}

impl Logger {
    pub fn new(pool: &SqlitePool, source: &str) -> Self {
        Self { pool: pool.clone(), source: source.to_string() }
    }

    pub async fn tag(&self, tag: &str, msg: &str) {
        let ts = chrono::Utc::now().timestamp();
        let result = sqlx::query("INSERT INTO logs (ts, tag, source, msg) VALUES (?, ?, ?, ?)")
            .bind(ts)
            .bind(tag)
            .bind(&self.source)
            .bind(msg)
            .execute(&self.pool)
            .await;
        if let (Ok(r), Some(app)) = (result, APP.get()) {
            let _ = app.emit("log_entry", serde_json::json!({
                "id":     r.last_insert_rowid(),
                "ts":     ts,
                "tag":    tag,
                "source": &self.source,
                "msg":    msg,
            }));
        }
    }
}
