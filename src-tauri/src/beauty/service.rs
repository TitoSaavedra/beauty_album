use std::path::Path;

use sea_orm::DatabaseConnection;

use crate::core::errors::AppError;
use crate::db::repositories::{class_repo::ClassRepository, preset_repo::PresetRepository};

pub async fn get_all_classes(db: &DatabaseConnection) -> Result<Vec<serde_json::Value>, AppError> {
    Ok(ClassRepository::get_all_with_counts(db).await?)
}

pub async fn get_presets(
    db: &DatabaseConnection,
    presets_dir: &Path,
    class_name: &str,
    sort_by: &str,
    search: &str,
    offset: i64,
    limit: i64,
) -> Result<Vec<serde_json::Value>, AppError> {
    Ok(PresetRepository::get_presets(db, presets_dir, class_name, sort_by, search, offset, limit).await?)
}

pub async fn get_popular_presets(
    db: &DatabaseConnection,
    popular_dir: &Path,
    presets_dir: &Path,
    class_name: &str,
    sort_by: &str,
    search: &str,
    since_ts: i64,
    offset: i64,
    limit: i64,
    wanted_ids: Vec<String>,
    region: &str,
) -> Result<Vec<serde_json::Value>, AppError> {
    let mut result = PresetRepository::get_popular_presets(
        db, popular_dir, presets_dir, class_name, sort_by, search, since_ts, offset, limit, region,
    )
    .await?;

    if !wanted_ids.is_empty() {
        result.sort_by(|a, b| {
            let a_w = wanted_ids.contains(&a["preset_id"].as_str().unwrap_or("").to_string());
            let b_w = wanted_ids.contains(&b["preset_id"].as_str().unwrap_or("").to_string());
            match (b_w, a_w) {
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                _ => std::cmp::Ordering::Equal,
            }
        });
    }

    Ok(result)
}

pub async fn get_popular_stats(
    db: &DatabaseConnection,
    class_name: &str,
) -> Result<serde_json::Value, AppError> {
    Ok(PresetRepository::get_popular_stats(db, class_name).await?)
}
