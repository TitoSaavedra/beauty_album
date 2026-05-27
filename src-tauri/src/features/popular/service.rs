use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use sea_orm::DatabaseConnection;
use tauri::AppHandle;

use crate::core::errors::AppError;
use crate::core::logger::Logger;
use crate::core::events::{self, ScrapperProgress};
use crate::db::repositories::{class_repo::ClassRepository, preset_repo::PresetRepository};
use crate::features::scraping::{garmoth::GarmothClient, image, browser};

pub const DAYS_ALL: &[&str] = &["20", "30", "60", "90", "180", "365", "ever"];
const REGIONS: &[&str] = &["eu", "na", "ru", "jp", "kr", "tw", "sa", "sea", "asia", "mena"];

async fn fetch_all(
    client: Arc<GarmothClient>,
    display_map: &std::collections::HashMap<u32, String>,
    log: &Logger,
) -> (std::collections::HashMap<u64, serde_json::Value>, usize) {
    let mut seen: std::collections::HashMap<u64, serde_json::Value> = std::collections::HashMap::new();
    let mut total_raw = 0usize;

    let mut handles = Vec::new();
    for &days in DAYS_ALL {
        let c = Arc::clone(&client);
        let d = days.to_string();
        handles.push((days, tokio::spawn(async move { c.fetch_popular(None, &d, "all").await })));
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
    log.tag("POP", &format!("Phase 1 done — unique so far: {}", seen.len())).await;

    let class_ids: Vec<(u32, String)> = display_map.iter().map(|(&id, n)| (id, n.clone())).collect();
    let mut handles2 = Vec::new();
    for (garmoth_id, class_name) in &class_ids {
        for &days in DAYS_ALL {
            let c = Arc::clone(&client);
            let d = days.to_string();
            let gid = *garmoth_id;
            let label = format!("{}/{}", class_name, days);
            handles2.push((label, tokio::spawn(async move { c.fetch_popular(Some(gid), &d, "all").await })));
        }
    }
    let mut phase2_raw = 0usize;
    for (label, handle) in handles2 {
        match handle.await {
            Ok(Ok(items)) => {
                phase2_raw += items.len();
                total_raw += items.len();
                if !items.is_empty() { log.tag("POP", &format!("{} → {} results", label, items.len())).await; }
                for item in items {
                    if let Some(id) = item["id"].as_u64() { seen.entry(id).or_insert(item); }
                }
            }
            Ok(Err(e)) => log.tag("ERR", &format!("{} failed: {}", label, e)).await,
            Err(e)     => log.tag("ERR", &format!("{} panicked: {}", label, e)).await,
        }
    }
    log.tag("POP", &format!("Phase 2 done — raw: {}  unique so far: {}", phase2_raw, seen.len())).await;

    let mut handles3 = Vec::new();
    for (garmoth_id, class_name) in &class_ids {
        for &days in DAYS_ALL {
            for &region in REGIONS {
                let c = Arc::clone(&client);
                let d = days.to_string();
                let r = region.to_string();
                let gid = *garmoth_id;
                let label = format!("{}/{}/{}", class_name, days, region);
                handles3.push((label, tokio::spawn(async move { c.fetch_popular(Some(gid), &d, &r).await })));
            }
        }
    }
    let mut phase3_raw = 0usize;
    for (label, handle) in handles3 {
        match handle.await {
            Ok(Ok(items)) => {
                phase3_raw += items.len();
                total_raw += items.len();
                if !items.is_empty() { log.tag("POP", &format!("{} → {} results", label, items.len())).await; }
                for item in items {
                    if let Some(id) = item["id"].as_u64() { seen.entry(id).or_insert(item); }
                }
            }
            Ok(Err(e)) => log.tag("ERR", &format!("{} failed: {}", label, e)).await,
            Err(e)     => log.tag("ERR", &format!("{} panicked: {}", label, e)).await,
        }
    }
    log.tag("POP", &format!("Phase 3 done — raw: {}  unique total: {}", phase3_raw, seen.len())).await;

    (seen, total_raw)
}

async fn insert_pending(
    db: &DatabaseConnection,
    items: &[serde_json::Value],
    log: &Logger,
    now: i64,
) -> usize {
    let mut inserted = 0usize;
    for item in items {
        let id = match item["id"].as_i64() { Some(i) => i, None => continue };
        let class_id = match item["class"].as_i64() { Some(i) => i, None => continue };
        let raw_json = serde_json::to_string(item).unwrap_or_default();

        let ok = PresetRepository::popular_insert(
            db,
            id,
            class_id,
            item["title"].as_str(),
            item["user_nickname"].as_str(),
            item["character_name"].as_str(),
            item["downloads"].as_i64().unwrap_or(0),
            item["views"].as_i64().unwrap_or(0),
            item["likes"].as_i64().unwrap_or(0),
            item["created_at"].as_i64(),
            item["customizing_id"].as_i64(),
            item["region"].as_str(),
            item["score"].as_i64(),
            now,
            &raw_json,
        )
        .await
        .is_ok();

        if ok { inserted += 1; }
    }
    log.tag("POP", &format!("Inserted {} pending presets", inserted)).await;
    inserted
}

async fn download_all(
    app: &AppHandle,
    browser_session: &browser::BrowserSession,
    db: &DatabaseConnection,
    popular_dir: &Path,
    log: &Logger,
    items: Vec<serde_json::Value>,
    display_map: &std::collections::HashMap<u32, String>,
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
            class_id,
            current: i + 1,
            total,
        });

        let Some(img) = img_name else {
            let _ = PresetRepository::popular_update_no_image_ok(db, id, chrono::Local::now().timestamp()).await;
            n_done += 1;
            let count = PresetRepository::count_ok_popular(db, class_id).await;
            events::emit_class_count_updated(app, class_id, count, true);
            continue;
        };

        let dest = preset_dir.join(img);
        let before_copy = image::find_image_in_dirs(&[popular_dir], id as u64, img).is_some();
        let got = image::acquire_image(
            browser_session,
            &[popular_dir],
            id as u64,
            class_id,
            img,
            &dest,
            log,
            class_display,
        )
        .await;

        if got {
            let _ = PresetRepository::popular_update_image_ok(db, id, img, chrono::Local::now().timestamp()).await;
            n_done += 1;
            if before_copy { n_copy += 1; } else { n_download += 1; }
            log.tag(class_display, &format!("Done [{}/{}] preset {}", i + 1, total, id)).await;
            let count = PresetRepository::count_ok_popular(db, class_id).await;
            events::emit_class_count_updated(app, class_id, count, true);
        } else {
            n_err += 1;
        }

        events::emit_popular_progress(app, ScrapperProgress {
            preset_id: id.to_string(),
            status: "done".to_string(),
            message: format!("Synced {}", id),
            class_name: class_display.to_string(),
            class_id,
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
    db: &DatabaseConnection,
    popular_dir: &Path,
    cancel: Arc<AtomicBool>,
) -> Result<String, AppError> {
    let log = Logger::new(db, "popular_service");
    log.tag("POP", "─── Starting popular sync ───").await;

    let display_map = ClassRepository::get_display_map(db).await?;
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

    let synced_ids = PresetRepository::popular_get_synced_ids(db).await?;

    let unique: Vec<serde_json::Value> = seen
        .into_values()
        .filter(|item| !synced_ids.contains(&(item["id"].as_u64().unwrap_or(0) as i64)))
        .collect();
    let already_done = synced_ids.len();

    let pending_rows = PresetRepository::popular_get_pending(db).await.unwrap_or_default();
    let unique_ids: std::collections::HashSet<u64> =
        unique.iter().filter_map(|i| i["id"].as_u64()).collect();
    let extra: Vec<serde_json::Value> = pending_rows
        .into_iter()
        .filter(|item| item["id"].as_u64().map_or(false, |id| !unique_ids.contains(&id)))
        .collect();

    let to_download: Vec<serde_json::Value> = unique.iter().cloned().chain(extra).collect();
    let total = to_download.len();

    log.tag("POP", &format!("Raw: {}  Already synced: {}  New: {}  To download: {}",
        total_raw, already_done, unique.len(), total)).await;

    if total == 0 {
        log.tag("POP", "All presets already synced").await;
        return Ok(format!("Popular already up to date — {} presets", already_done));
    }

    let now = chrono::Local::now().timestamp();
    let inserted = insert_pending(db, &unique, &log, now).await;
    if inserted == 0 && unique.is_empty() {
        log.tag("WARN", "No new presets from API — retrying pending only").await;
    }

    log.tag("POP", "Starting browser session").await;
    let browser_session = match browser::BrowserSession::new().await {
        Ok(s) => { log.tag("POP", "Browser session ready").await; s }
        Err(e) => {
            log.tag("ERR", &format!("Browser session failed: {}", e)).await;
            return Err(e);
        }
    };

    let (n_done, n_copy, n_download, n_err) =
        download_all(app, &browser_session, db, popular_dir, &log, to_download, &display_map, cancel, total).await;

    let msg = format!(
        "Popular sync done — {} synced ({} copied, {} downloaded)  {} already done  {} error(s)",
        n_done, n_copy, n_download, already_done, n_err
    );
    log.tag("POP", &msg).await;
    Ok(msg)
}
