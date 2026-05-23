use std::path::Path;

use chrono::{TimeZone, Utc};
use sqlx::{Row, SqlitePool};

use crate::errors::AppError;

const ALBUM_GET_CLASSES_PRESETS: &str = include_str!("SQL/album_get_classes_presets.sql");
const ALBUM_GET_CLASSES_POPULAR: &str = include_str!("SQL/album_get_classes_popular.sql");
const ALBUM_GET_PRESETS: &str = include_str!("SQL/album_get_presets.sql");
const ALBUM_GET_POPULAR_PRESETS: &str = include_str!("SQL/album_get_popular_presets.sql");

pub async fn get_classes_for_presets(pool: &SqlitePool) -> Result<Vec<serde_json::Value>, AppError> {
    get_classes_query(pool, ALBUM_GET_CLASSES_PRESETS).await
}

pub async fn get_classes_for_popular(pool: &SqlitePool) -> Result<Vec<serde_json::Value>, AppError> {
    get_classes_query(pool, ALBUM_GET_CLASSES_POPULAR).await
}

async fn get_classes_query(pool: &SqlitePool, sql: &str) -> Result<Vec<serde_json::Value>, AppError> {
    let rows = sqlx::query(sql).fetch_all(pool).await?;

    let mut classes = Vec::new();
    for row in rows {
        let display: String = row.get("display");
        let icon_svg: Option<String> = row.get("icon_svg");
        let is_favorite: i64 = row.get("is_favorite");
        let preset_count: i64 = row.get("preset_count");

        classes.push(serde_json::json!({
            "name": display,
            "icon_svg": icon_svg,
            "preset_count": preset_count,
            "is_favorite": is_favorite == 1,
        }));
    }

    Ok(classes)
}

pub async fn get_presets(
    pool: &SqlitePool,
    presets_dir: &Path,
    class_name: &str,
    sort_by: &str,
    offset: i64,
    limit: i64,
) -> Result<Vec<serde_json::Value>, AppError> {
    let order_col = match sort_by {
        "views" => "views",
        "favorites" => "likes",
        _ => "downloads",
    };

    let sql = ALBUM_GET_PRESETS.replace("{order_col}", order_col);
    let rows = sqlx::query(&sql)
        .bind(class_name)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let mut result = Vec::new();
    for row in rows {
        let id: i64 = row.get("id");
        let id_str = id.to_string();
        let image_1: Option<String> = row.get("image_1");
        let image_2: Option<String> = row.get("image_2");
        let pab_file: Option<String> = row.get("pab_file");
        let created_at: Option<i64> = row.get("created_at");
        let updated_at: Option<i64> = row.get("updated_at");
        let downloads: i64 = row.get("downloads");
        let views: i64 = row.get("views");
        let likes: i64 = row.get("likes");
        let title: Option<String> = row.get("title");
        let user_nickname: Option<String> = row.get("user_nickname");
        let character_name: Option<String> = row.get("character_name");

        let preset_dir = presets_dir.join(class_name).join(&id_str);

        let mut image_paths: Vec<String> = Vec::new();
        for img in [&image_1, &image_2].into_iter().flatten() {
            let p = preset_dir.join(img);
            if p.exists() {
                image_paths.push(p.to_string_lossy().replace('\\', "/"));
            }
        }

        let download_path: Option<String> = pab_file.as_deref().and_then(|pab| {
            let p = preset_dir.join(pab);
            p.exists().then(|| p.to_string_lossy().replace('\\', "/").to_string())
        });

        let date = created_at.and_then(|ts| {
            if let chrono::LocalResult::Single(dt) = Utc.timestamp_opt(ts, 0) {
                Some(dt.format("%Y-%m-%d").to_string())
            } else {
                None
            }
        });

        result.push(serde_json::json!({
            "preset_id": id_str,
            "id": id,
            "title": title,
            "creator": user_nickname,
            "char_name": character_name,
            "downloads": downloads,
            "views": views,
            "likes": likes,
            "favorites": likes,
            "image_paths": image_paths,
            "download_path": download_path,
            "creation_at": created_at,
            "updated_at": updated_at,
            "date": date,
        }));
    }

    Ok(result)
}

pub async fn get_popular_presets(
    pool: &SqlitePool,
    popular_dir: &Path,
    presets_dir: &Path,
    class_name: &str,
    sort_by: &str,
    offset: i64,
    limit: i64,
) -> Result<Vec<serde_json::Value>, AppError> {
    let order_col = match sort_by {
        "views" => "views",
        "favorites" => "likes",
        _ => "downloads",
    };

    let sql = ALBUM_GET_POPULAR_PRESETS.replace("{order_col}", order_col);
    let rows = sqlx::query(&sql)
        .bind(class_name)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let mut result = Vec::new();
    for row in rows {
        let id: i64 = row.get("id");
        let id_str = id.to_string();
        let image_1: Option<String> = row.get("image_1");
        let is_wanted: i64 = row.get("is_wanted");
        let is_downloaded: i64 = row.get("is_downloaded");
        let downloads: i64 = row.get("downloads");
        let views: i64 = row.get("views");
        let likes: i64 = row.get("likes");
        let title: Option<String> = row.get("title");
        let user_nickname: Option<String> = row.get("user_nickname");
        let updated_at: Option<i64> = row.get("updated_at");

        let popular_preset_dir = popular_dir.join(class_name).join(&id_str);
        let image_paths: Vec<String> = image_1.as_deref().map(|img| {
            let p_popular = popular_preset_dir.join(img);
            if p_popular.exists() {
                return vec![p_popular.to_string_lossy().replace('\\', "/")];
            }
            let p_presets = presets_dir.join(class_name).join(&id_str).join(img);
            if p_presets.exists() {
                vec![p_presets.to_string_lossy().replace('\\', "/")]
            } else {
                vec![]
            }
        }).unwrap_or_default();

        result.push(serde_json::json!({
            "preset_id": id_str,
            "id": id,
            "title": title,
            "creator": user_nickname,
            "downloads": downloads,
            "views": views,
            "likes": likes,
            "favorites": likes,
            "image_paths": image_paths,
            "download_path": null,
            "updated_at": updated_at,
            "is_popular": true,
            "is_downloaded": is_downloaded == 1,
            "is_wanted": is_wanted == 1,
        }));
    }

    Ok(result)
}
