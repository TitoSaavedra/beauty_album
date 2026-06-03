use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use sea_orm::DatabaseConnection;
use tauri::{AppHandle, Manager};

use crate::core::errors::AppError;
use crate::core::logger::Logger;
use crate::core::state::{AppState};
use crate::core::events::{self, ScrapperProgress, ProgressStatus, ProgressType};
use crate::db::repositories::{class_repo::ClassRepository, preset_repo::PresetRepository};
use crate::beauty::scrapping::{garmoth::GarmothClient, browser, image, progress_service::ProgressContext};

const DAYS_ALL: &[&str] = &["20", "30", "60", "90", "180", "365", "ever"];
const REGIONS: &[&str] = &["eu", "na", "ru", "jp", "kr", "tw", "sa", "sea", "asia", "mena"];

enum PresetType {
    Personal,
    Popular,
}


struct PendingPreset {
    preset_id: u64,
    class_hint: String,
    pab_path: PathBuf,
}

fn parse_pab_filename(filename: &str) -> Option<(String, u64)> {
    let stem = filename.strip_suffix(".pab").unwrap_or(filename);
    let underscore = stem.find('_')?;
    let class_hint = stem[..underscore].to_string();
    let id_pos = stem.rfind("ID")?;
    let after_id = &stem[id_pos + 2..];
    let digits: String = after_id.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() { return None; }
    let id: u64 = digits.parse().ok()?;
    Some((class_hint, id))
}

fn scan_input(input_dir: &Path) -> Vec<PendingPreset> {
    let mut result = Vec::new();
    scan_recursive(input_dir, &mut result);
    result
}

fn scan_recursive(dir: &Path, result: &mut Vec<PendingPreset>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_recursive(&path, result);
        } else if path.is_file() {
            let filename = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if let Some((class_hint, preset_id)) = parse_pab_filename(&filename) {
                result.push(PendingPreset { preset_id, class_hint, pab_path: path });
            }
        }
    }
}

async fn download_preset_images(
    app: &AppHandle,
    browser: &browser::BrowserSession,
    db: &DatabaseConnection,
    popular_dir: &Path,
    log: &Logger,
    preset_id: u64,
    class_id: u32,
    class_name: String,
    image_1: Option<String>,
    image_2: Option<String>,
    dest_dir: &Path,
    current: usize,
    total: usize,
    preset_type: PresetType,
) -> (bool, bool) {
    let ctx = match preset_type {
        PresetType::Personal => ProgressContext::new_preset(app, preset_id.to_string(), class_name.clone(), class_id, current, total),
        PresetType::Popular => ProgressContext::new_popular(app, preset_id.to_string(), class_name.clone(), class_id, current, total),
    };

    if image_1.is_none() && image_2.is_none() {
        ctx.emit_done(format!("Synced {}", preset_id));
        return (true, false);
    }

    ctx.emit_processing(format!("Downloading {}", preset_id));

    let mut primary_img: Option<&str> = None;
    let mut img1_exists_before = false;
    let mut img1_success = false;

    if let Some(img1) = &image_1 {
        img1_exists_before = image::find_image_in_dirs(&[popular_dir], preset_id, img1).is_some();
        let dest = dest_dir.join(img1);
        img1_success = image::acquire_image(
            browser,
            &[popular_dir],
            preset_id,
            class_id,
            img1,
            &dest,
            log,
            &class_name,
        )
        .await;
        if img1_success {
            primary_img = Some(img1);
        }
    }

    if !img1_success {
        if let Some(img2) = &image_2 {
            let dest = dest_dir.join(img2);
            let img2_success = image::acquire_image(
                browser,
                &[popular_dir],
                preset_id,
                class_id,
                img2,
                &dest,
                log,
                &class_name,
            )
            .await;
            if img2_success {
                primary_img = Some(img2);
            }
        }
    }

    let now = chrono::Local::now().timestamp();
    match primary_img {
        Some(img) => {
            match preset_type {
                PresetType::Personal => {
                    let img2_saved = if img == image_1.as_deref().unwrap_or("") {
                        if let Some(img2) = &image_2 {
                            image::acquire_image(browser, &[popular_dir], preset_id, class_id, img2, &dest_dir.join(img2), log, &class_name).await
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if img2_saved {
                        let _ = PresetRepository::update_image_both(db, preset_id, img, image_2.as_deref().unwrap(), now).await;
                    } else {
                        let _ = PresetRepository::update_image_one(db, preset_id, img, now).await;
                    }
                }
                PresetType::Popular => {
                    let _ = PresetRepository::popular_update_image_ok(db, preset_id as i64, img, now).await;
                }
            }

            log.tag(&class_name, &format!("Done preset {}", preset_id)).await;
            ctx.emit_done(format!("Synced {}", preset_id));
            (true, !img1_exists_before)
        }
        None => {
            match preset_type {
                PresetType::Personal => {
                    ctx.emit_done(format!("Synced {} (no image)", preset_id));
                }
                PresetType::Popular => {
                    let _ = PresetRepository::popular_update_no_image_ok(db, preset_id as i64, now).await;
                    ctx.emit_done(format!("Synced {}", preset_id));
                }
            }
            (true, false)
        }
    }
}
pub async fn orchestrate_scraping(
    app: &AppHandle,
    db: &DatabaseConnection,
    input_dir: &Path,
    presets_dir: &Path,
    popular_dir: &Path,
    cancel: Arc<AtomicBool>,
    skip_fetch: bool,
) -> Result<String, AppError> {
    let log = Logger::new(db, "orchestrator");

    events::emit_sync_loading(app, "Initializing sync...");

    log.tag("ORCH", "─── Phase 1: Personal Presets ───").await;
    let personal_result = run_personal(app, db, input_dir, presets_dir, popular_dir, cancel.clone(), skip_fetch).await?;
    log.tag("ORCH", &personal_result).await;

    if cancel.load(Ordering::Relaxed) {
        log.tag("ORCH", "Cancelled by user").await;
        return Ok("Scraping cancelled".to_string());
    }

    log.tag("ORCH", "─── Phase 2: Popular Presets ───").await;
    let popular_result = run_popular(app, db, popular_dir, cancel).await?;
    log.tag("ORCH", &popular_result).await;

    events::emit_progress(app, ScrapperProgress {
        preset_id: "0".to_string(),
        status: ProgressStatus::Done,
        message: "Sync complete".to_string(),
        class_name: "SYNC".to_string(),
        class_id: 0,
        current: 0,
        total: 0,
        progress_type: ProgressType::Preset,
    });

    let combined = format!("✓ {}\n✓ {}", personal_result, popular_result);
    Ok(combined)
}

pub async fn run_personal(
    app: &AppHandle,
    db: &DatabaseConnection,
    input_dir: &Path,
    presets_dir: &Path,
    popular_dir: &Path,
    cancel: Arc<AtomicBool>,
    skip_fetch: bool,
) -> Result<String, AppError> {
    let log = Logger::new(db, "personal_filter");

    let all_presets = scan_input(input_dir);
    log.tag("PERSONAL", &format!("Scanned: {} preset(s)", all_presets.len())).await;

    let mut pending = Vec::new();
    for preset in all_presets {
        let is_pending = !PresetRepository::ok_exists(db, preset.preset_id).await;
        if is_pending {
            pending.push(preset);
        }
    }

    log.tag("PERSONAL", &format!("Pending (is_ok=0): {} preset(s)", pending.len())).await;

    if pending.is_empty() {
        return Ok("No pending presets".to_string());
    }

    let cf_clearance = {
        let state = app.state::<AppState>();
        let guard = state.0.lock().map_err(|e| AppError::Scrape(e.to_string()))?;
        guard.cf_clearance.clone()
    };

    let client = Arc::new(GarmothClient::new(&cf_clearance));
    let display_map = ClassRepository::get_display_map(db).await?;

    struct SavedPreset {
        preset_id: u64,
        class_hint: String,
        class_id: u32,
        pab_path: PathBuf,
        preset_dir: PathBuf,
        image_1: Option<String>,
        image_2: Option<String>,
    }

    let mut saved_presets = Vec::new();

    let total = pending.len();

    log.tag("PERSONAL", "Starting browser session").await;
    let browser = Arc::new(match browser::BrowserSession::new().await {
        Ok(s) => { log.tag("PERSONAL", "Browser session ready").await; s }
        Err(e) => {
            log.tag("ERR", &format!("Browser session failed: {}", e)).await;
            return Err(e);
        }
    });

    let mut download_handles = Vec::new();

    if !skip_fetch {
        log.tag("PERSONAL", "─── Fetching & Downloading ───").await;
        for (i, preset) in pending.into_iter().enumerate() {
            if cancel.load(Ordering::Relaxed) {
                log.tag("PERSONAL", "Fetch cancelled by user").await;
                break;
            }

            let (data, raw) = match client.fetch_preset(preset.preset_id).await {
                Ok(d) => d,
                Err(e) => {
                    log.tag("PERSONAL", &format!("Fetch error {}: {}", preset.preset_id, e)).await;
                    continue;
                }
            };

            if cancel.load(Ordering::Relaxed) {
                log.tag("PERSONAL", "Fetch cancelled by user").await;
                break;
            }

            let mut image_1 = data.image_1.as_deref().filter(|s| !s.is_empty()).map(String::from);
            let mut image_2 = data.image_2.as_deref().filter(|s| !s.is_empty()).map(String::from);
            if image_1.is_none() && image_2.is_some() {
                image_1 = image_2.take();
            }

            let raw_json = serde_json::to_string(&raw).unwrap_or_default();
            let now = chrono::Local::now().timestamp();

            let _ = PresetRepository::insert_preset(
                db,
                data.id as i64,
                data.class as i64,
                data.title.as_deref(),
                data.user_nickname.as_deref(),
                data.character_name.as_deref(),
                data.downloads as i64,
                data.views as i64,
                data.likes as i64,
                image_2.as_deref(),
                &format!("{}.pab", preset.preset_id),
                Some(data.creation_at),
                now,
                &raw_json,
            ).await;

            let preset_dir = presets_dir.join(&preset.class_hint).join(preset.preset_id.to_string());
            let class_id = display_map.get(&(data.class as u32)).map(|_| data.class as u32).unwrap_or(0);

            events::emit_progress(app, ScrapperProgress {
                preset_id: preset.preset_id.to_string(),
                status: ProgressStatus::Metadata,
                message: format!("Ready: {}", preset.preset_id),
                class_name: preset.class_hint.clone(),
                class_id,
                current: i + 1,
                total,
                progress_type: ProgressType::Preset,
            });

            let app_h = app.clone();
            let db_h = db.clone();
            let browser_h = Arc::clone(&browser);
            let popular_dir_h = popular_dir.to_path_buf();
            let preset_dir_h = preset_dir.clone();
            let pab_path = preset.pab_path.clone();
            let class_hint = preset.class_hint.clone();
            let preset_id = preset.preset_id;

            let handle = tokio::spawn(async move {
                let _ = tokio::fs::create_dir_all(&preset_dir_h).await;
                let (done, _) = download_preset_images(
                    &app_h,
                    &browser_h,
                    &db_h,
                    &popular_dir_h,
                    &Logger::new(&db_h, "personal_download"),
                    preset_id,
                    class_id,
                    class_hint.clone(),
                    image_1,
                    image_2,
                    &preset_dir_h,
                    i + 1,
                    total,
                    PresetType::Personal,
                ).await;

                if done {
                    let pab_filename = pab_path.file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    let _ = tokio::fs::copy(&pab_path, preset_dir_h.join(&pab_filename)).await;
                    let _ = PresetRepository::mark_ok(&db_h, preset_id).await;
                }
            });

            download_handles.push(handle);
        }
    } else {
        log.tag("PERSONAL", "─── Phase 1: Skip (using existing data) ───").await;
        for preset in pending {
            let preset_dir = presets_dir.join(&preset.class_hint).join(preset.preset_id.to_string());
            saved_presets.push(SavedPreset {
                preset_id: preset.preset_id,
                class_hint: preset.class_hint,
                class_id: 0,
                pab_path: preset.pab_path,
                preset_dir,
                image_1: None,
                image_2: None,
            });
        }

        let total_skip = saved_presets.len();
        for (i, saved) in saved_presets.into_iter().enumerate() {
            if cancel.load(Ordering::Relaxed) {
                break;
            }

            let _ = tokio::fs::create_dir_all(&saved.preset_dir).await;

            let (done, _) = download_preset_images(
                app,
                &browser,
                db,
                popular_dir,
                &log,
                saved.preset_id,
                saved.class_id,
                saved.class_hint.clone(),
                saved.image_1,
                saved.image_2,
                &saved.preset_dir,
                i + 1,
                total_skip,
                PresetType::Personal,
            ).await;

            if done {
                let pab_filename = saved.pab_path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let _ = tokio::fs::copy(&saved.pab_path, saved.preset_dir.join(&pab_filename)).await;
                let _ = PresetRepository::mark_ok(db, saved.preset_id).await;
            }
        }
    }

    for handle in download_handles {
        let _ = handle.await;
    }

    let msg = format!("Personal sync done — {} queued", total);
    log.tag("PERSONAL", &msg).await;
    Ok(msg)
}

async fn fetch_all(
    client: Arc<GarmothClient>,
    display_map: &std::collections::HashMap<u32, String>,
    log: &Logger,
    cancel: Arc<AtomicBool>,
) -> (std::collections::HashMap<u64, serde_json::Value>, usize) {
    let mut seen: std::collections::HashMap<u64, serde_json::Value> = std::collections::HashMap::new();
    let mut total_raw = 0usize;

    let mut handles = Vec::new();
    for &days in DAYS_ALL {
        if cancel.load(Ordering::Relaxed) {
            log.tag("POP", "Fetch cancelled by user").await;
            return (seen, total_raw);
        }
        let c = Arc::clone(&client);
        let d = days.to_string();
        handles.push((days, tokio::spawn(async move { c.fetch_popular(None, &d, "all").await })));
    }
    for (days, handle) in handles {
        if cancel.load(Ordering::Relaxed) {
            log.tag("POP", "Fetch cancelled by user").await;
            return (seen, total_raw);
        }
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
        if cancel.load(Ordering::Relaxed) {
            log.tag("POP", "Fetch cancelled by user").await;
            return (seen, total_raw);
        }
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
        if cancel.load(Ordering::Relaxed) {
            log.tag("POP", "Fetch cancelled by user").await;
            return (seen, total_raw);
        }
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
        if cancel.load(Ordering::Relaxed) {
            log.tag("POP", "Fetch cancelled by user").await;
            return (seen, total_raw);
        }
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
        if cancel.load(Ordering::Relaxed) {
            log.tag("POP", "Fetch cancelled by user").await;
            return (seen, total_raw);
        }
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
            item["creation_at"].as_i64(),
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

async fn download_all_parallel(
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
    let mut n_done = 0;
    let mut n_copy = 0;
    let mut n_download = 0;
    let mut n_err = 0;

    for (i, item) in items.into_iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            log.tag("USER", "Popular sync cancelled").await;
            break;
        }

        let id = match item["id"].as_i64() { Some(v) => v, None => continue };
        let class_id = item["class"].as_u64().unwrap_or(0) as u32;
        let class_display = display_map.get(&class_id).map(|s| s.as_str()).unwrap_or("Unknown").to_string();

        let preset_dir = popular_dir.join(&class_display).join(id.to_string());
        let _ = tokio::fs::create_dir_all(&preset_dir).await;

        let image_1 = item["image_1"].as_str().filter(|s| !s.is_empty()).map(String::from);
        let image_2 = item["image_2"].as_str().filter(|s| !s.is_empty()).map(String::from);

        let before_copy = image_1.as_deref()
            .or(image_2.as_deref())
            .map(|img| image::find_image_in_dirs(&[popular_dir], id as u64, img).is_some())
            .unwrap_or(false);

        events::emit_progress(app, ScrapperProgress {
            preset_id: id.to_string(),
            status: ProgressStatus::Processing,
            message: format!("downloading assets {} {}/{}", class_display, i + 1, total),
            class_name: class_display.clone(),
            class_id,
            current: i + 1,
            total,
            progress_type: ProgressType::Popular,
        });

        let (done, _http_dl) = download_preset_images(
            app,
            browser_session,
            db,
            popular_dir,
            log,
            id as u64,
            class_id,
            class_display,
            image_1,
            image_2,
            &preset_dir,
            i + 1,
            total,
            PresetType::Popular,
        ).await;

        if done {
            n_done += 1;
            if before_copy { n_copy += 1; } else { n_download += 1; }
        } else {
            n_err += 1;
        }
    }

    (n_done, n_copy, n_download, n_err)
}

pub async fn run_popular(
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

    log.tag("POP", "─── Phase 1: Fetch & Save ───").await;
    let client = Arc::new(GarmothClient::new(""));
    let (seen, total_raw) = fetch_all(client, &display_map, &log, cancel.clone()).await;

    if seen.is_empty() {
        log.tag("POP", "Nothing to sync — all windows empty").await;
        return Ok("No popular presets found".to_string());
    }

    let ok_preset_ids = PresetRepository::get_ok_preset_ids(db).await.unwrap_or_default();
    let synced_ids = PresetRepository::popular_get_synced_ids(db).await?;


    let now = chrono::Local::now().timestamp();
    for (preset_id, raw_item) in &seen {
        if ok_preset_ids.contains(&(*preset_id as i64)) {
            let _ = PresetRepository::popular_update_details(
                db,
                *preset_id as i64,
                raw_item["title"].as_str(),
                raw_item["user_nickname"].as_str(),
                raw_item["character_name"].as_str(),
                raw_item["downloads"].as_i64().unwrap_or(0),
                raw_item["views"].as_i64().unwrap_or(0),
                raw_item["likes"].as_i64().unwrap_or(0),
                raw_item["creation_at"].as_i64(),
                raw_item["customizing_id"].as_i64(),
                raw_item["region"].as_str(),
                raw_item["score"].as_i64(),
                &serde_json::to_string(raw_item).unwrap_or_default(),
                now,
            ).await;
        }
    }

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

    let inserted = insert_pending(db, &unique, &log, now).await;
    if inserted == 0 && unique.is_empty() {
        log.tag("WARN", "No new presets from API — retrying pending only").await;
    }

    events::emit_progress(app, ScrapperProgress {
        preset_id: "0".to_string(),
        status: ProgressStatus::Processing,
        message: format!("Ready to download {} presets", total),
        class_name: "POPULAR".to_string(),
        class_id: 0,
        current: 0,
        total,
        progress_type: ProgressType::Popular,
    });

    log.tag("POP", "─── Phase 2: Check images in DB ───").await;
    let mut to_download_images = PresetRepository::popular_get_missing_images(db).await?;

    // Sort: top 10 by downloads per class first, then rest
    {
        use std::collections::HashMap;
        let mut by_class: HashMap<u32, Vec<serde_json::Value>> = HashMap::new();
        for item in to_download_images.drain(..) {
            let class = item["class"].as_u64().unwrap_or(0) as u32;
            by_class.entry(class).or_insert_with(Vec::new).push(item);
        }

        let mut top_10_all = Vec::new();
        let mut rest = Vec::new();

        for (_, mut items) in by_class {
            items.sort_by_key(|item| std::cmp::Reverse(item["downloads"].as_i64().unwrap_or(0)));
            let (top, bottom) = items.split_at(items.len().min(10));
            top_10_all.extend(top.iter().cloned());
            rest.extend(bottom.iter().cloned());
        }

        to_download_images = top_10_all;
        to_download_images.extend(rest);
    }

    let total_to_download = to_download_images.len();
    log.tag("POP", &format!("Presets needing images: {}", total_to_download)).await;

    if total_to_download == 0 {
        log.tag("POP", "All presets have images").await;
        let msg = format!("Popular sync done — {} synced ({} copied, {} downloaded)  {} already done  0 error(s)",
            0, 0, 0, already_done);
        log.tag("POP", &msg).await;
        return Ok(msg);
    }

    events::emit_progress(app, ScrapperProgress {
        preset_id: "0".to_string(),
        status: ProgressStatus::Processing,
        message: format!("Starting download of {} images", total_to_download),
        class_name: "POPULAR".to_string(),
        class_id: 0,
        current: 0,
        total: total_to_download,
        progress_type: ProgressType::Popular,
    });

    log.tag("POP", "─── Phase 3: Download Images ───").await;
    log.tag("POP", "Starting browser session").await;
    let browser_session = match browser::BrowserSession::new().await {
        Ok(s) => { log.tag("POP", "Browser session ready").await; s }
        Err(e) => {
            log.tag("ERR", &format!("Browser session failed: {}", e)).await;
            return Err(e);
        }
    };

    let (n_done, n_copy, n_download, n_err) =
        download_all_parallel(app, &browser_session, db, popular_dir, &log, to_download_images, &display_map, cancel, total_to_download).await;

    let msg = format!(
        "Popular sync done — {} synced ({} copied, {} downloaded)  {} already done  {} error(s)",
        n_done, n_copy, n_download, already_done, n_err
    );
    log.tag("POP", &msg).await;
    Ok(msg)
}

