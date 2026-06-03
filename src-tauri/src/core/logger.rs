use sea_orm::DatabaseConnection;

use crate::db::repositories::log_repo::LogRepository;

pub struct Logger {
    db: DatabaseConnection,
    source: String,
}

impl Logger {
    pub fn new(db: &DatabaseConnection, source: &str) -> Self {
        Self { db: db.clone(), source: source.to_string() }
    }

    pub async fn tag(&self, tag: &str, msg: &str) {
        let ts = chrono::Utc::now().timestamp();
        let _ = LogRepository::insert(&self.db, ts, tag, &self.source, msg).await;
    }
}
