use std::collections::HashSet;
use std::path::Path;

use chrono::{TimeZone, Utc};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr, Statement, TransactionTrait, Value};
use serde_json;

pub struct PresetRepository;

impl PresetRepository {
    // ─── Album queries ───────────────────────────────────────────────────────

    pub async fn get_presets(
        db: &impl ConnectionTrait,
        presets_dir: &Path,
        class_name: &str,
        sort_by: &str,
        search: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, DbErr> {
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

        let sql = format!(
            "SELECT p.id, p.title, p.user_nickname, p.character_name,
                    p.downloads, p.views, p.likes, p.image_1, p.image_2, p.pab_file, p.created_at, p.updated_at
             FROM presets p
             JOIN classes c ON c.id_garmoth = p.class_id
             WHERE c.display = ?
               AND p.is_ok IN (0, 1)
               AND p.is_popular = 0
               AND (? = '' OR p.title LIKE ? OR p.user_nickname LIKE ?)
             ORDER BY p.{order_col} DESC
             LIMIT ? OFFSET ?"
        );

        let rows = db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                &sql,
                [
                    class_name.into(),
                    search.into(),
                    search_like.as_str().into(),
                    search_like.as_str().into(),
                    limit.into(),
                    offset.into(),
                ],
            ))
            .await?;

        let mut result = Vec::new();
        for r in &rows {
            let id: i64 = r.try_get("", "id").unwrap_or(0);
            let id_str = id.to_string();
            let image_1: Option<String> = r.try_get("", "image_1").unwrap_or(None);
            let image_2: Option<String> = r.try_get("", "image_2").unwrap_or(None);
            let pab_file: Option<String> = r.try_get("", "pab_file").unwrap_or(None);
            let created_at: Option<i64> = r.try_get("", "created_at").unwrap_or(None);
            let updated_at: Option<i64> = r.try_get("", "updated_at").unwrap_or(None);

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
            let date = format_date(created_at);

            result.push(serde_json::json!({
                "preset_id":     id_str,
                "id":            id,
                "title":         r.try_get::<Option<String>>("", "title").unwrap_or(None),
                "creator":       r.try_get::<Option<String>>("", "user_nickname").unwrap_or(None),
                "char_name":     r.try_get::<Option<String>>("", "character_name").unwrap_or(None),
                "downloads":     r.try_get::<i64>("", "downloads").unwrap_or(0),
                "views":         r.try_get::<i64>("", "views").unwrap_or(0),
                "likes":         r.try_get::<i64>("", "likes").unwrap_or(0),
                "favorites":     r.try_get::<i64>("", "likes").unwrap_or(0),
                "image_paths":   image_paths,
                "download_path": download_path,
                "created_at":    created_at,
                "updated_at":    updated_at,
                "date":          date,
            }));
        }
        Ok(result)
    }

    pub async fn get_popular_presets(
        db: &impl ConnectionTrait,
        popular_dir: &Path,
        presets_dir: &Path,
        class_name: &str,
        sort_by: &str,
        search: &str,
        since_ts: i64,
        offset: i64,
        limit: i64,
        region: &str,
    ) -> Result<Vec<serde_json::Value>, DbErr> {
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

        let sql = format!(
            "SELECT p.id, p.title, p.user_nickname, p.downloads, p.views, p.likes,
                    p.image_1, p.is_wanted, p.created_at, p.updated_at,
                    EXISTS(SELECT 1 FROM presets p2 WHERE p2.id = p.id AND p2.is_popular = 0 AND p2.is_ok = 1) AS is_downloaded
             FROM presets p
             JOIN classes c ON c.id_garmoth = p.class_id
             WHERE c.display = ?
               AND p.is_popular = 1
               AND p.is_discarded = 0
               AND p.is_ok = 1
               AND (? = 0 OR p.created_at >= ?)
               AND (? = '' OR p.title LIKE ? OR p.user_nickname LIKE ?)
               AND (? = '' OR p.region = ?)
             ORDER BY p.{order_col} DESC
             LIMIT ? OFFSET ?"
        );

        let rows = db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                &sql,
                [
                    class_name.into(),
                    since_ts.into(),
                    since_ts.into(),
                    search.into(),
                    search_like.as_str().into(),
                    search_like.as_str().into(),
                    region.into(),
                    region.into(),
                    limit.into(),
                    offset.into(),
                ],
            ))
            .await?;

        let mut result = Vec::new();
        for r in &rows {
            let id: i64 = r.try_get("", "id").unwrap_or(0);
            let id_str = id.to_string();
            let image_1: Option<String> = r.try_get("", "image_1").unwrap_or(None);
            let created_at: Option<i64> = r.try_get("", "created_at").unwrap_or(None);

            let popular_preset_dir = popular_dir.join(class_name).join(&id_str);
            let image_paths: Vec<String> = image_1
                .as_deref()
                .map(|img| {
                    let p = popular_preset_dir.join(img);
                    if p.exists() {
                        return vec![p.to_string_lossy().replace('\\', "/")];
                    }
                    let p2 = presets_dir.join(class_name).join(&id_str).join(img);
                    if p2.exists() {
                        vec![p2.to_string_lossy().replace('\\', "/")]
                    } else {
                        vec![]
                    }
                })
                .unwrap_or_default();

            result.push(serde_json::json!({
                "preset_id":    id_str,
                "id":           id,
                "title":        r.try_get::<Option<String>>("", "title").unwrap_or(None),
                "creator":      r.try_get::<Option<String>>("", "user_nickname").unwrap_or(None),
                "downloads":    r.try_get::<i64>("", "downloads").unwrap_or(0),
                "views":        r.try_get::<i64>("", "views").unwrap_or(0),
                "likes":        r.try_get::<i64>("", "likes").unwrap_or(0),
                "favorites":    r.try_get::<i64>("", "likes").unwrap_or(0),
                "image_paths":  image_paths,
                "download_path": null,
                "date":         format_date(created_at),
                "updated_at":   r.try_get::<Option<i64>>("", "updated_at").unwrap_or(None),
                "is_popular":   true,
                "is_downloaded": r.try_get::<i64>("", "is_downloaded").unwrap_or(0) == 1,
                "is_wanted":    r.try_get::<i64>("", "is_wanted").unwrap_or(0) == 1,
            }));
        }
        Ok(result)
    }

    pub async fn get_popular_stats(
        db: &impl ConnectionTrait,
        class_name: &str,
    ) -> Result<serde_json::Value, DbErr> {
        let row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "SELECT
                   COUNT(*) AS total,
                   SUM(CASE WHEN p.created_at >= strftime('%s', 'now') - 20*86400  THEN 1 ELSE 0 END) AS d20,
                   SUM(CASE WHEN p.created_at >= strftime('%s', 'now') - 30*86400  THEN 1 ELSE 0 END) AS d30,
                   SUM(CASE WHEN p.created_at >= strftime('%s', 'now') - 60*86400  THEN 1 ELSE 0 END) AS d60,
                   SUM(CASE WHEN p.created_at >= strftime('%s', 'now') - 90*86400  THEN 1 ELSE 0 END) AS d90,
                   SUM(CASE WHEN p.created_at >= strftime('%s', 'now') - 180*86400 THEN 1 ELSE 0 END) AS d180,
                   SUM(CASE WHEN p.created_at >= strftime('%s', 'now') - 365*86400 THEN 1 ELSE 0 END) AS d365
                 FROM presets p
                 JOIN classes c ON c.id_garmoth = p.class_id
                 WHERE c.display = ? AND p.is_popular = 1 AND p.is_discarded = 0 AND p.is_ok = 1",
                [class_name.into()],
            ))
            .await?;

        Ok(serde_json::json!({
            "total": row.as_ref().and_then(|r| r.try_get::<i64>("", "total").ok()).unwrap_or(0),
            "d20":   row.as_ref().and_then(|r| r.try_get::<i64>("", "d20").ok()).unwrap_or(0),
            "d30":   row.as_ref().and_then(|r| r.try_get::<i64>("", "d30").ok()).unwrap_or(0),
            "d60":   row.as_ref().and_then(|r| r.try_get::<i64>("", "d60").ok()).unwrap_or(0),
            "d90":   row.as_ref().and_then(|r| r.try_get::<i64>("", "d90").ok()).unwrap_or(0),
            "d180":  row.as_ref().and_then(|r| r.try_get::<i64>("", "d180").ok()).unwrap_or(0),
            "d365":  row.as_ref().and_then(|r| r.try_get::<i64>("", "d365").ok()).unwrap_or(0),
        }))
    }

    pub async fn get_preset_by_id(
        db: &impl ConnectionTrait,
        popular_dir: &Path,
        presets_dir: &Path,
        id: i64,
    ) -> Result<Option<serde_json::Value>, DbErr> {
        let row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "SELECT p.id, p.title, p.user_nickname, p.downloads, p.views, p.likes,
                        p.image_1, p.is_wanted, p.created_at, p.updated_at,
                        c.display AS class_display
                 FROM presets p
                 JOIN classes c ON c.id_garmoth = p.class_id
                 WHERE p.id = ? AND p.is_popular = 1 AND p.is_discarded = 0",
                [id.into()],
            ))
            .await?;

        let Some(r) = row else { return Ok(None) };

        let id_str = id.to_string();
        let class_display: String = r.try_get("", "class_display").unwrap_or_default();
        let image_1: Option<String> = r.try_get("", "image_1").unwrap_or(None);
        let image_paths: Vec<String> = image_1
            .as_deref()
            .map(|img| {
                let p = popular_dir.join(&class_display).join(&id_str).join(img);
                if p.exists() {
                    return vec![p.to_string_lossy().replace('\\', "/")];
                }
                let p2 = presets_dir.join(&class_display).join(&id_str).join(img);
                if p2.exists() {
                    vec![p2.to_string_lossy().replace('\\', "/")]
                } else {
                    vec![]
                }
            })
            .unwrap_or_default();

        Ok(Some(serde_json::json!({
            "preset_id":    id_str,
            "id":           id,
            "title":        r.try_get::<Option<String>>("", "title").unwrap_or(None),
            "creator":      r.try_get::<Option<String>>("", "user_nickname").unwrap_or(None),
            "downloads":    r.try_get::<i64>("", "downloads").unwrap_or(0),
            "views":        r.try_get::<i64>("", "views").unwrap_or(0),
            "favorites":    r.try_get::<i64>("", "likes").unwrap_or(0),
            "image_paths":  image_paths,
            "download_path": null,
            "date":         null,
            "updated_at":   r.try_get::<Option<i64>>("", "updated_at").unwrap_or(None),
            "is_popular":   true,
            "is_downloaded": false,
            "is_wanted":    r.try_get::<i64>("", "is_wanted").unwrap_or(0) == 1,
            "class_display": class_display,
        })))
    }

    // ─── Scraper operations ──────────────────────────────────────────────────

    pub async fn ok_exists(db: &impl ConnectionTrait, id: u64) -> bool {
        db.query_one(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "SELECT EXISTS(SELECT 1 FROM presets WHERE id=? AND is_ok=1 AND is_popular=0) AS e",
            [(id as i64).into()],
        ))
        .await
        .ok()
        .flatten()
        .and_then(|r| r.try_get::<i64>("", "e").ok())
        .map(|v| v == 1)
        .unwrap_or(false)
    }

    pub async fn reset_preset(
        db: &impl ConnectionTrait,
        id: u64,
        pab_file: &str,
        now: i64,
    ) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "UPDATE presets SET is_popular=0, is_ok=0, pab_file=?, updated_at=? WHERE id=?",
            [pab_file.into(), now.into(), (id as i64).into()],
        ))
        .await?;
        Ok(())
    }

    pub async fn insert_preset(
        db: &impl ConnectionTrait,
        id: i64,
        class_id: i64,
        title: Option<&str>,
        user_nickname: Option<&str>,
        character_name: Option<&str>,
        downloads: i64,
        views: i64,
        likes: i64,
        image_2: Option<&str>,
        pab_file: &str,
        created_at: Option<i64>,
        updated_at: i64,
        raw_json: &str,
    ) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "INSERT OR REPLACE INTO presets
                (id, class_id, title, user_nickname, character_name, downloads, views, likes,
                 image_2, pab_file, created_at, updated_at, is_ok, is_popular, raw_json)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0, ?)",
            [
                id.into(),
                class_id.into(),
                title.map(Value::from).unwrap_or(Value::String(None)),
                user_nickname.map(Value::from).unwrap_or(Value::String(None)),
                character_name.map(Value::from).unwrap_or(Value::String(None)),
                downloads.into(),
                views.into(),
                likes.into(),
                image_2.map(Value::from).unwrap_or(Value::String(None)),
                pab_file.into(),
                created_at.map(Value::from).unwrap_or(Value::BigInt(None)),
                updated_at.into(),
                raw_json.into(),
            ],
        ))
        .await?;
        Ok(())
    }

    pub async fn update_image_both(
        db: &impl ConnectionTrait,
        id: u64,
        img1: &str,
        img2: &str,
        now: i64,
    ) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "UPDATE presets SET image_1=?, image_2=?, is_ok=1, updated_at=? WHERE id=?",
            [img1.into(), img2.into(), now.into(), (id as i64).into()],
        ))
        .await?;
        Ok(())
    }

    pub async fn update_image_one(
        db: &impl ConnectionTrait,
        id: u64,
        img1: &str,
        now: i64,
    ) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "UPDATE presets SET image_1=?, is_ok=1, updated_at=? WHERE id=?",
            [img1.into(), now.into(), (id as i64).into()],
        ))
        .await?;
        Ok(())
    }

    pub async fn set_is_ok(db: &impl ConnectionTrait, id: u64, now: i64) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "UPDATE presets SET is_ok=1, updated_at=? WHERE id=?",
            [now.into(), (id as i64).into()],
        ))
        .await?;
        Ok(())
    }

    // ─── Popular sync operations ─────────────────────────────────────────────

    pub async fn popular_get_synced_ids(
        db: &impl ConnectionTrait,
    ) -> Result<HashSet<i64>, DbErr> {
        let rows = db
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT id FROM presets WHERE is_ok = 1 AND is_popular = 1".to_string(),
            ))
            .await?;
        Ok(rows
            .iter()
            .filter_map(|r| r.try_get::<i64>("", "id").ok())
            .collect())
    }

    pub async fn popular_insert(
        db: &impl ConnectionTrait,
        id: i64,
        class_id: i64,
        title: Option<&str>,
        user_nickname: Option<&str>,
        character_name: Option<&str>,
        downloads: i64,
        views: i64,
        likes: i64,
        created_at: Option<i64>,
        customizing_id: Option<i64>,
        region: Option<&str>,
        score: Option<i64>,
        now: i64,
        raw_json: &str,
    ) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "INSERT OR IGNORE INTO presets
                (id, class_id, title, user_nickname, character_name, downloads, views, likes,
                 created_at, customizing_id, region, score, is_ok, is_popular, updated_at, raw_json)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 1, ?, ?)",
            [
                id.into(),
                class_id.into(),
                title.map(Value::from).unwrap_or(Value::String(None)),
                user_nickname.map(Value::from).unwrap_or(Value::String(None)),
                character_name.map(Value::from).unwrap_or(Value::String(None)),
                downloads.into(),
                views.into(),
                likes.into(),
                created_at.map(Value::from).unwrap_or(Value::BigInt(None)),
                customizing_id.map(Value::from).unwrap_or(Value::BigInt(None)),
                region.map(Value::from).unwrap_or(Value::String(None)),
                score.map(Value::from).unwrap_or(Value::BigInt(None)),
                now.into(),
                raw_json.into(),
            ],
        ))
        .await?;
        Ok(())
    }

    pub async fn popular_update_image_ok(
        db: &impl ConnectionTrait,
        id: i64,
        img1: &str,
        now: i64,
    ) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "UPDATE presets SET image_1 = ?, is_ok = 1, updated_at = ? WHERE id = ?",
            [img1.into(), now.into(), id.into()],
        ))
        .await?;
        Ok(())
    }

    pub async fn popular_update_no_image_ok(
        db: &impl ConnectionTrait,
        id: i64,
        now: i64,
    ) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "UPDATE presets SET is_ok = 1, updated_at = ? WHERE id = ?",
            [now.into(), id.into()],
        ))
        .await?;
        Ok(())
    }

    pub async fn popular_get_pending(
        db: &impl ConnectionTrait,
    ) -> Result<Vec<serde_json::Value>, DbErr> {
        let rows = db
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT raw_json FROM presets WHERE is_popular = 1 AND is_ok = 0 AND raw_json IS NOT NULL"
                    .to_string(),
            ))
            .await?;
        Ok(rows
            .iter()
            .filter_map(|r| {
                r.try_get::<String>("", "raw_json")
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
            })
            .collect())
    }

    // ─── Wanted / discard ────────────────────────────────────────────────────

    pub async fn wanted_get_ids(db: &impl ConnectionTrait) -> Result<Vec<i64>, DbErr> {
        let rows = db
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT id FROM presets WHERE is_wanted = 1 AND is_popular = 1".to_string(),
            ))
            .await?;
        Ok(rows
            .iter()
            .filter_map(|r| r.try_get::<i64>("", "id").ok())
            .collect())
    }

    pub async fn wanted_set_bulk(
        db: &DatabaseConnection,
        ids: &[i64],
    ) -> Result<(), DbErr> {
        let txn = db.begin().await?;
        txn.execute(Statement::from_string(
            DbBackend::Sqlite,
            "UPDATE presets SET is_wanted = 0 WHERE is_popular = 1".to_string(),
        ))
        .await?;
        for &id in ids {
            txn.execute(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "UPDATE presets SET is_wanted = 1 WHERE id = ? AND is_popular = 1",
                [id.into()],
            ))
            .await?;
        }
        txn.commit().await?;
        Ok(())
    }

    pub async fn wanted_get(db: &impl ConnectionTrait, id: i64) -> Result<i64, DbErr> {
        let row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                "SELECT is_wanted FROM presets WHERE id = ? AND is_popular = 1",
                [id.into()],
            ))
            .await?;
        Ok(row
            .and_then(|r| r.try_get::<i64>("", "is_wanted").ok())
            .unwrap_or(0))
    }

    pub async fn wanted_toggle(
        db: &impl ConnectionTrait,
        id: i64,
        new_val: i64,
    ) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "UPDATE presets SET is_wanted = ? WHERE id = ? AND is_popular = 1",
            [new_val.into(), id.into()],
        ))
        .await?;
        Ok(())
    }

    pub async fn discard(db: &impl ConnectionTrait, id: i64) -> Result<(), DbErr> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "UPDATE presets SET is_discarded = 1 WHERE id = ? AND is_popular = 1",
            [id.into()],
        ))
        .await?;
        Ok(())
    }

    pub async fn get_regions(db: &impl ConnectionTrait) -> Result<Vec<String>, DbErr> {
        let rows = db
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT DISTINCT region FROM presets WHERE is_popular = 1 AND region IS NOT NULL ORDER BY region"
                    .to_string(),
            ))
            .await?;
        Ok(rows
            .iter()
            .filter_map(|r| r.try_get::<String>("", "region").ok())
            .collect())
    }
}

fn format_date(ts: Option<i64>) -> Option<String> {
    ts.and_then(|t| {
        if let chrono::LocalResult::Single(dt) = Utc.timestamp_opt(t, 0) {
            Some(dt.format("%Y-%m-%d").to_string())
        } else {
            None
        }
    })
}
