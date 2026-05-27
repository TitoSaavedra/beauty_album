use std::sync::OnceLock;

use sea_orm::DatabaseConnection;
use tauri::AppHandle;

use crate::core::events;
use crate::db::repositories::log_repo::LogRepository;

static APP: OnceLock<AppHandle> = OnceLock::new();

pub fn init(app: &AppHandle) {
    let _ = APP.set(app.clone());
}

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
        let result = LogRepository::insert(&self.db, ts, tag, &self.source, msg).await;
        if let (Ok(row_id), Some(app)) = (result, APP.get()) {
            events::emit_log_entry(app, row_id, ts, tag, &self.source, msg);
        }
    }
}
