use sqlx::SqlitePool;

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
        let _ = sqlx::query("INSERT INTO logs (ts, tag, source, msg) VALUES (?, ?, ?, ?)")
            .bind(ts)
            .bind(tag)
            .bind(&self.source)
            .bind(msg)
            .execute(&self.pool)
            .await;
    }
}
