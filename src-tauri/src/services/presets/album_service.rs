use std::path::Path;

use chrono::{TimeZone, Utc};
use sqlx::{Row, SqlitePool};

use crate::core::errors::AppError;

const ALBUM_GET_CLASSES_PRESETS: &str = include_str!("sql/album_get_classes_presets.sql");
const ALBUM_GET_CLASSES_POPULAR: &str = include_str!("sql/album_get_classes_popular.sql");
const ALBUM_GET_PRESETS: &str = include_str!("sql/album_get_presets.sql");
const ALBUM_GET_POPULAR_PRESETS: &str = include_str!("sql/album_get_popular_presets.sql");
const ALBUM_GET_POPULAR_STATS: &str = include_str!("sql/album_get_popular_stats.sql");

pub async fn get_classes_for_presets(pool: &SqlitePool) -> Result<Vec<serde_json::Value>, AppError> {
    get_classes_query(pool, ALBUM_GET_CLASSES_PRESETS).await
}

pub async fn get_classes_for_popular(pool: &SqlitePool, since_ts: i64) -> Result<Vec<serde_json::Value>, AppError> {
    let rows = sqlx::query(ALBUM_GET_CLASSES_POPULAR)
        .bind(since_ts)
        .bind(since_ts)
        .fetch_all(pool)
        .await?;

    let mut classes = Vec::new();
    for row in rows {
        let id_garmoth: i64 = row.get("id_garmoth");
        let display: String = row.get("display");
        let icon_svg: Option<String> = row.get("icon_svg");
        let is_favorite: i64 = row.get("is_favorite");
        let preset_count: i64 = row.get("preset_count");

        classes.push(serde_json::json!({
            "class_id": id_garmoth,
            "name": display,
            "icon_svg": icon_svg,
            "preset_count": preset_count,
            "is_favorite": is_favorite == 1,
        }));
    }

    Ok(classes)
}

async fn get_classes_query(pool: &SqlitePool, sql: &str) -> Result<Vec<serde_json::Value>, AppError> {
    let rows = sqlx::query(sql).fetch_all(pool).await?;

    let mut classes = Vec::new();
    for row in rows {
        let id_garmoth: i64 = row.get("id_garmoth");
        let display: String = row.get("display");
        let icon_svg: Option<String> = row.get("icon_svg");
        let is_favorite: i64 = row.get("is_favorite");
        let preset_count: i64 = row.get("preset_count");

        classes.push(serde_json::json!({
            "class_id": id_garmoth,
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
    search: &str,
    offset: i64,
    limit: i64,
) -> Result<Vec<serde_json::Value>, AppError> {
    let order_col = match sort_by {
        "views" => "views",
        "favorites" => "likes",
        _ => "downloads",
    };

    let search_like = if search.is_empty() {
        String::new()
    } else {
        format!("%{}%", search)
    };

    let sql = ALBUM_GET_PRESETS.replace("{order_col}", order_col);
    let rows = sqlx::query(&sql)
        .bind(class_name)
        .bind(search)
        .bind(&search_like)
        .bind(&search_like)
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
            "created_at": created_at,
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
    search: &str,
    since_ts: i64,
    offset: i64,
    limit: i64,
) -> Result<Vec<serde_json::Value>, AppError> {
    let order_col = match sort_by {
        "views" => "views",
        "favorites" => "likes",
        _ => "downloads",
    };

    let search_like = if search.is_empty() {
        String::new()
    } else {
        format!("%{}%", search)
    };

    let sql = ALBUM_GET_POPULAR_PRESETS.replace("{order_col}", order_col);
    let rows = sqlx::query(&sql)
        .bind(class_name)
        .bind(since_ts)
        .bind(since_ts)
        .bind(search)
        .bind(&search_like)
        .bind(&search_like)
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
        let created_at: Option<i64> = row.get("created_at");
        let updated_at: Option<i64> = row.get("updated_at");

        let date = created_at.and_then(|ts| {
            if let chrono::LocalResult::Single(dt) = Utc.timestamp_opt(ts, 0) {
                Some(dt.format("%Y-%m-%d").to_string())
            } else {
                None
            }
        });

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
            "date": date,
            "updated_at": updated_at,
            "is_popular": true,
            "is_downloaded": is_downloaded == 1,
            "is_wanted": is_wanted == 1,
        }));
    }

    Ok(result)
}

pub async fn get_popular_stats(
    pool: &SqlitePool,
    class_name: &str,
) -> Result<serde_json::Value, AppError> {
    let row = sqlx::query(ALBUM_GET_POPULAR_STATS)
        .bind(class_name)
        .fetch_one(pool)
        .await?;

    Ok(serde_json::json!({
        "total": row.get::<i64, _>("total"),
        "d20":   row.get::<i64, _>("d20"),
        "d30":   row.get::<i64, _>("d30"),
        "d60":   row.get::<i64, _>("d60"),
        "d90":   row.get::<i64, _>("d90"),
        "d180":  row.get::<i64, _>("d180"),
        "d365":  row.get::<i64, _>("d365"),
    }))
}
