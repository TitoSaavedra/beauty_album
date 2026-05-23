use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use sqlx::{Row, SqlitePool};
use tauri::AppHandle;

use crate::errors::AppError;
use crate::services::{
    events,
    log::Logger,
    scraping::{garmoth_client::GarmothClient, image, playwright_service},
};
use crate::services::scraping::scrapper_service::ScrapperProgress;

const POPULAR_GET_SYNCED_IDS: &str = include_str!("SQL/popular_get_synced_ids.sql");
const POPULAR_INSERT_PRESET: &str = include_str!("SQL/popular_insert_preset.sql");
const POPULAR_UPDATE_IMAGE_OK: &str = include_str!("SQL/popular_update_image_ok.sql");
const POPULAR_UPDATE_NO_IMAGE_OK: &str = include_str!("SQL/popular_update_no_image_ok.sql");

pub const DAYS_ALL: &[&str] = &["20", "30", "60", "90", "180", "365", "ever"];
pub const DAYS_PER_CLASS: &[&str] = &["180", "365", "ever"];

/// Loads id_garmoth → display name map for directory/log display only.
async fn load_display_map(pool: &SqlitePool) -> HashMap<u32, String> {
    sqlx::query("SELECT id_garmoth, display FROM classes")
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .iter()
        .map(|r| {
            let id: i64 = r.get(0);
            let display: String = r.get(1);
            (id as u32, display)
        })
        .collect()
}

/// Fetches all time-window results concurrently and deduplicates by ID.
async fn fetch_all(
    client: Arc<GarmothClient>,
    display_map: &HashMap<u32, String>,
    log: &Logger,
) -> (HashMap<u64, serde_json::Value>, usize) {
    let mut seen: HashMap<u64, serde_json::Value> = HashMap::new();
    let mut total_raw = 0usize;

    let mut handles = Vec::new();
    for &days in DAYS_ALL {
        let c = Arc::clone(&client);
        let d = days.to_string();
        handles.push((days, tokio::spawn(async move { c.fetch_popular(None, &d).await })));
    }
    for (days, handle) in handles {
        match handle.await {
            Ok(Ok(items)) => {
                total_raw += items.len();
                log.tag("POP", &format!("all/{} → {} results", days, items.len())).await;
                for item in items {
                    if let Some(id) = item["id"].as_u64() { seen.entry(id).or_insert(item); }
                }
            }
            Ok(Err(e)) => log.tag("ERR", &format!("all/{} failed: {}", days, e)).await,
            Err(e)     => log.tag("ERR", &format!("all/{} panicked: {}", days, e)).await,
        }
    }

    let class_ids: Vec<(u32, String)> = display_map.iter().map(|(&id, n)| (id, n.clone())).collect();
    let mut handles2 = Vec::new();
    for (garmoth_id, class_name) in &class_ids {
        for &days in DAYS_PER_CLASS {
            let c = Arc::clone(&client);
            let d = days.to_string();
            let gid = *garmoth_id;
            let label = format!("{}/{}", class_name, days);
            handles2.push((label, tokio::spawn(async move { c.fetch_popular(Some(gid), &d).await })));
        }
    }
    let mut phase2_raw = 0usize;
    for (label, handle) in handles2 {
        match handle.await {
            Ok(Ok(items)) => {
                phase2_raw += items.len();
                total_raw += items.len();
                if !items.is_empty() {
                    log.tag("POP", &format!("{} → {} results", label, items.len())).await;
                }
                for item in items {
                    if let Some(id) = item["id"].as_u64() { seen.entry(id).or_insert(item); }
                }
            }
            Ok(Err(e)) => log.tag("ERR", &format!("{} failed: {}", label, e)).await,
            Err(e)     => log.tag("ERR", &format!("{} panicked: {}", label, e)).await,
        }
    }
    log.tag("POP", &format!("Phase 2 done — raw: {}  unique: {}", phase2_raw, seen.len())).await;

    (seen, total_raw)
}

/// Inserts pending presets individually (no transaction) — class_id from JSON directly.
async fn insert_pending(
    pool: &SqlitePool,
    items: &[serde_json::Value],
    log: &Logger,
    now: i64,
) -> usize {
    let mut inserted = 0usize;
    for item in items {
        let id = match item["id"].as_i64() { Some(i) => i, None => continue };
        let class_id = match item["class"].as_i64() { Some(i) => i, None => continue };
        let raw_json = serde_json::to_string(item).unwrap_or_default();

        let ok = sqlx::query(POPULAR_INSERT_PRESET)
            .bind(id)
            .bind(class_id)
            .bind(item["title"].as_str())
            .bind(item["user_nickname"].as_str())
            .bind(item["character_name"].as_str())
            .bind(item["downloads"].as_i64().unwrap_or(0))
            .bind(item["views"].as_i64().unwrap_or(0))
            .bind(item["likes"].as_i64().unwrap_or(0))
            .bind(item["created_at"].as_i64())
            .bind(item["customizing_id"].as_i64())
            .bind(item["region"].as_str())
            .bind(item["score"].as_i64())
            .bind(now)
            .bind(&raw_json)
            .execute(pool)
            .await
            .is_ok();

        if ok { inserted += 1; }
    }
    log.tag("POP", &format!("Inserted {} pending presets", inserted)).await;
    inserted
}

/// Downloads images for all pending presets.
/// Returns (n_done, n_copy, n_download, n_err).
async fn download_all(
    app: &AppHandle,
    browser: &playwright_service::BrowserSession,
    pool: &SqlitePool,
    popular_dir: &Path,
    log: &Logger,
    items: Vec<serde_json::Value>,
    display_map: &HashMap<u32, String>,
    cancel: Arc<AtomicBool>,
    total: usize,
) -> (usize, usize, usize, usize) {
    let (mut n_done, mut n_copy, mut n_download, mut n_err) = (0, 0, 0, 0);

    for (i, item) in items.into_iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            log.tag("USER", "Popular sync cancelled").await;
            break;
        }

        let id = match item["id"].as_i64() { Some(v) => v, None => continue };
        let class_id = item["class"].as_u64().unwrap_or(0) as u32;
        let class_display = display_map.get(&class_id).map(|s| s.as_str()).unwrap_or("Unknown");

        let preset_dir = popular_dir.join(class_display).join(id.to_string());
        let _ = tokio::fs::create_dir_all(&preset_dir).await;

        let image_1 = item["image_1"].as_str().filter(|s| !s.is_empty()).map(String::from);
        let image_2 = item["image_2"].as_str().filter(|s| !s.is_empty()).map(String::from);
        let img_name = image_1.as_deref().or(image_2.as_deref());

        events::emit_popular_progress(app, ScrapperProgress {
            preset_id: id.to_string(),
            status: "metadata".to_string(),
            message: format!("Ready: {}", id),
            class_name: class_display.to_string(),
            current: i + 1,
            total,
        });

        let Some(img) = img_name else {
            let _ = sqlx::query(POPULAR_UPDATE_NO_IMAGE_OK)
                .bind(chrono::Local::now().timestamp())
                .bind(id)
                .execute(pool)
                .await;
            n_done += 1;
            continue;
        };

        let dest = preset_dir.join(img);
        let before_copy = image::find_image_in_dirs(&[popular_dir], id as u64, img).is_some();
        let got = image::acquire_image(
            browser,
            &[popular_dir],
            id as u64,
            class_id,
            img,
            &dest,
            log,
            class_display,
        ).await;

        if got {
            let _ = sqlx::query(POPULAR_UPDATE_IMAGE_OK)
                .bind(img)
                .bind(chrono::Local::now().timestamp())
                .bind(id)
                .execute(pool)
                .await;
            n_done += 1;
            if before_copy { n_copy += 1; } else { n_download += 1; }
            log.tag(class_display, &format!("Done [{}/{}] preset {}", i + 1, total, id)).await;
        } else {
            n_err += 1;
        }

        events::emit_popular_progress(app, ScrapperProgress {
            preset_id: id.to_string(),
            status: "done".to_string(),
            message: format!("Synced {}", id),
            class_name: class_display.to_string(),
            current: i + 1,
            total,
        });

        if !before_copy && got {
            image::jitter_sleep().await;
        }
    }

    (n_done, n_copy, n_download, n_err)
}

pub async fn sync_popular(
    app: &AppHandle,
    pool: &SqlitePool,
    popular_dir: &Path,
    cancel: Arc<AtomicBool>,
) -> Result<String, AppError> {
    let log = Logger::new(pool, "popular_service");
    log.tag("POP", "─── Starting popular sync ───").await;

    let display_map = load_display_map(pool).await;
    log.tag("POP", &format!("Classes loaded: {}", display_map.len())).await;
    if display_map.is_empty() {
        let msg = "Class map not found — run class init first";
        log.tag("ERR", msg).await;
        return Err(AppError::Scrape(msg.to_string()));
    }

    let client = Arc::new(GarmothClient::new(""));
    let (seen, total_raw) = fetch_all(client, &display_map, &log).await;

    if seen.is_empty() {
        log.tag("POP", "Nothing to sync — all windows empty").await;
        return Ok("No popular presets found".to_string());
    }

    let synced_ids: std::collections::HashSet<i64> = {
        let rows = sqlx::query(POPULAR_GET_SYNCED_IDS).fetch_all(pool).await?;
        rows.iter().map(|r| r.get::<i64, _>(0)).collect()
    };

    let unique: Vec<serde_json::Value> = seen.into_values()
        .filter(|item| !synced_ids.contains(&(item["id"].as_u64().unwrap_or(0) as i64)))
        .collect();
    let already_done = synced_ids.len();
    let total = unique.len();

    log.tag("POP", &format!("Raw: {}  Already synced: {}  To process: {}", total_raw, already_done, total)).await;

    if total == 0 {
        log.tag("POP", "All presets already synced").await;
        return Ok(format!("Popular already up to date — {} presets", already_done));
    }

    let now = chrono::Local::now().timestamp();
    let inserted = insert_pending(pool, &unique, &log, now).await;
    if inserted == 0 {
        log.tag("WARN", "No presets inserted — DB may have issues, attempting download of existing pending").await;
    }

    log.tag("POP", "Starting browser session").await;
    let browser = match playwright_service::BrowserSession::new().await {
        Ok(s) => { log.tag("POP", "Browser session ready").await; s }
        Err(e) => {
            log.tag("ERR", &format!("Browser session failed: {}", e)).await;
            return Err(e);
        }
    };

    let (n_done, n_copy, n_download, n_err) =
        download_all(app, &browser, pool, popular_dir, &log, unique, &display_map, cancel, total).await;

    let msg = format!(
        "Popular sync done — {} synced ({} copied, {} downloaded)  {} already done  {} error(s)",
        n_done, n_copy, n_download, already_done, n_err
    );
    log.tag("POP", &msg).await;
    Ok(msg)
}
