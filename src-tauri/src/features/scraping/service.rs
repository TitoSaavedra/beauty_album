use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use sqlx::{Row, SqlitePool};
use tauri::{AppHandle, Manager};

use crate::core::errors::AppError;
use crate::core::logger::Logger;
use crate::core::state::{AppState, DbPool, ScrapperCancelToken};
use crate::core::events::{self, ScrapperProgress};
use crate::features::scraping::{garmoth::{GarmothClient, GarmothPreset}, browser, image};

const UPDATE_PRESET_IS_OK: &str = include_str!("sql/scraper_update_is_ok.sql");
const SELECT_OK_EXISTS: &str = include_str!("sql/scraper_ok_exists.sql");
const SELECT_COUNT_OK: &str = include_str!("sql/scraper_count_ok.sql");
const UPDATE_IMAGE_BOTH: &str = include_str!("sql/scraper_update_image_both.sql");
const UPDATE_IMAGE_ONE: &str = include_str!("sql/scraper_update_image_one.sql");
const RESET_PRESET: &str = include_str!("sql/scraper_reset_preset.sql");
const INSERT_PRESET: &str = include_str!("sql/scraper_insert_preset.sql");

struct PendingPreset {
    preset_id: u64,
    class_hint: String,
    pab_path: PathBuf,
}

struct FetchedMeta {
    preset: PendingPreset,
    data: GarmothPreset,
    preset_dir: PathBuf,
    image_1: Option<String>,
    image_2: Option<String>,
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

async fn ok_exists_db(pool: &SqlitePool, preset_id: u64) -> bool {
    sqlx::query(SELECT_OK_EXISTS)
        .bind(preset_id as i64)
        .fetch_one(pool)
        .await
        .map(|row| row.get::<i64, _>(0) == 1)
        .unwrap_or(false)
}

pub async fn pending_count(pool: &SqlitePool, input_dir: &Path) -> usize {
    let pending = scan_input(input_dir);
    let mut count = 0;
    for p in &pending {
        if !ok_exists_db(pool, p.preset_id).await {
            count += 1;
        }
    }
    count
}

async fn process_image(
    meta: &FetchedMeta,
    browser: &browser::BrowserSession,
    pool: &SqlitePool,
    popular_dir: &Path,
    log: &Logger,
    app: &AppHandle,
    current: usize,
    total: usize,
) -> (bool, bool) {
    let Some(img1) = &meta.image_1 else {
        let _ = sqlx::query(UPDATE_PRESET_IS_OK)
            .bind(chrono::Local::now().timestamp())
            .bind(meta.preset.preset_id as i64)
            .execute(pool)
            .await;
        let count: i64 = sqlx::query_scalar(SELECT_COUNT_OK)
        .bind(meta.data.class as i64)
        .fetch_one(pool)
        .await
        .unwrap_or(0);
        events::emit_class_count_updated(app, meta.data.class, count, false);
        events::emit_progress(app, ScrapperProgress {
            preset_id: meta.preset.preset_id.to_string(),
            status: "done".to_string(),
            message: format!("Synced {}", meta.preset.preset_id),
            class_name: meta.preset.class_hint.clone(),
            class_id: meta.data.class,
            current,
            total,
        });
        return (true, false);
    };

    events::emit_progress(app, ScrapperProgress {
        preset_id: meta.preset.preset_id.to_string(),
        status: "processing".to_string(),
        message: format!("Downloading {}", meta.preset.preset_id),
        class_name: meta.preset.class_hint.clone(),
        class_id: meta.data.class,
        current,
        total,
    });

    let dest = meta.preset_dir.join(img1);
    let already_exists = image::find_image_in_dirs(&[popular_dir], meta.preset.preset_id, img1).is_some();
    let got = image::acquire_image(
        browser,
        &[popular_dir],
        meta.preset.preset_id,
        meta.data.class,
        img1,
        &dest,
        log,
        &meta.preset.class_hint,
    ).await;

    if !got {
        let _ = sqlx::query(UPDATE_PRESET_IS_OK)
            .bind(chrono::Local::now().timestamp())
            .bind(meta.preset.preset_id as i64)
            .execute(pool)
            .await;
        let count: i64 = sqlx::query_scalar(SELECT_COUNT_OK)
        .bind(meta.data.class as i64)
        .fetch_one(pool)
        .await
        .unwrap_or(0);
        events::emit_class_count_updated(app, meta.data.class, count, false);
        events::emit_progress(app, ScrapperProgress {
            preset_id: meta.preset.preset_id.to_string(),
            status: "done".to_string(),
            message: format!("Synced {} (no image)", meta.preset.preset_id),
            class_name: meta.preset.class_hint.clone(),
            class_id: meta.data.class,
            current,
            total,
        });
        return (true, false);
    }

    let img2_saved = if let Some(img2) = &meta.image_2 {
        image::acquire_image(browser, &[popular_dir], meta.preset.preset_id, meta.data.class, img2, &meta.preset_dir.join(img2), log, &meta.preset.class_hint).await
    } else {
        false
    };

    let now = chrono::Local::now().timestamp();
    if img2_saved {
        let _ = sqlx::query(UPDATE_IMAGE_BOTH)
            .bind(img1.as_str())
            .bind(meta.image_2.as_deref().unwrap())
            .bind(now)
            .bind(meta.preset.preset_id as i64)
            .execute(pool)
            .await;
    } else {
        let _ = sqlx::query(UPDATE_IMAGE_ONE)
            .bind(img1.as_str())
            .bind(now)
            .bind(meta.preset.preset_id as i64)
            .execute(pool)
            .await;
    }

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM presets WHERE class_id=? AND is_ok=1 AND is_popular=0"
    )
    .bind(meta.data.class as i64)
    .fetch_one(pool)
    .await
    .unwrap_or(0);
    events::emit_class_count_updated(app, meta.data.class, count, false);

    log.tag(&meta.preset.class_hint, &format!("Done preset {}", meta.preset.preset_id)).await;
    events::emit_progress(app, ScrapperProgress {
        preset_id: meta.preset.preset_id.to_string(),
        status: "done".to_string(),
        message: format!("Synced {}", meta.preset.preset_id),
        class_name: meta.preset.class_hint.clone(),
        class_id: meta.data.class,
        current,
        total,
    });

    (true, !already_exists)
}


pub async fn run(
    app: &AppHandle,
    pool: &SqlitePool,
    input_dir: &Path,
    presets_dir: &Path,
    popular_dir: &Path,
    cancel: Arc<AtomicBool>,
) -> Result<String, AppError> {
    let log = Logger::new(pool, "scrapper_service");

    let mut pending_all = scan_input(input_dir);
    let mut pending = Vec::new();
    for p in pending_all.drain(..) {
        if !ok_exists_db(pool, p.preset_id).await {
            pending.push(p);
        }
    }

    let total = pending.len();
    log.tag("SYNC", &format!("Starting: {} preset(s) pending", total)).await;
    if total == 0 {
        events::emit_scrapper_done(app, "no_pending");
        return Ok("No presets pending".to_string());
    }

    let cf_clearance = {
        let state = app.state::<AppState>();
        let guard = state.0.lock().map_err(|e| AppError::Scrape(e.to_string()))?;
        guard.cf_clearance.clone()
    };

    let client = Arc::new(GarmothClient::new(&cf_clearance));

    log.tag("SYNC", "Starting browser session").await;
    let browser = match browser::BrowserSession::new().await {
        Ok(s) => { log.tag("SYNC", "Browser session ready").await; s }
        Err(e) => {
            log.tag("ERR", &format!("Browser session failed: {}", e)).await;
            return Err(e);
        }
    };

    let (tx, mut rx) = tokio::sync::mpsc::channel::<FetchedMeta>(total);
    let fetch_errors = Arc::new(AtomicUsize::new(0));
    let n_meta = Arc::new(AtomicUsize::new(0));

    for (i, preset) in pending.into_iter().enumerate() {
        let client = Arc::clone(&client);
        let tx = tx.clone();
        let app_h = app.clone();
        let pool = pool.clone();
        let p_dir = presets_dir.to_path_buf();
        let cancel = Arc::clone(&cancel);
        let fetch_errors = Arc::clone(&fetch_errors);
        let n_meta = Arc::clone(&n_meta);

        tokio::spawn(async move {
            let log = Logger::new(&pool, "scrapper_service");

            if cancel.load(Ordering::Relaxed) {
                events::emit_progress(&app_h, ScrapperProgress {
                    preset_id: preset.preset_id.to_string(),
                    status: "cancelled".to_string(),
                    message: "Cancelled".to_string(),
                    class_name: preset.class_hint.clone(),
                    class_id: 0,
                    current: i + 1,
                    total,
                });
                return;
            }

            let preset_dir = p_dir.join(&preset.class_hint).join(preset.preset_id.to_string());
            let _ = tokio::fs::create_dir_all(&preset_dir).await;

            let pab_filename = preset.pab_path.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let _ = tokio::fs::copy(&preset.pab_path, preset_dir.join(&pab_filename)).await;

            let now_pre = chrono::Local::now().timestamp();
            let _ = sqlx::query(RESET_PRESET)
            .bind(&pab_filename)
            .bind(now_pre)
            .bind(preset.preset_id as i64)
            .execute(&pool)
            .await;

            events::emit_progress(&app_h, ScrapperProgress {
                preset_id: preset.preset_id.to_string(),
                status: "processing".to_string(),
                message: format!("Fetching {}", preset.preset_id),
                class_name: preset.class_hint.clone(),
                class_id: 0,
                current: i + 1,
                total,
            });

            let (data, raw) = match client.fetch_preset(preset.preset_id).await {
                Ok(d) => d,
                Err(e) => {
                    fetch_errors.fetch_add(1, Ordering::Relaxed);
                    log.tag(&preset.class_hint, &format!("ERR meta {}: {}", preset.preset_id, e)).await;
                    return;
                }
            };

            let mut image_1 = data.image_1.as_deref().filter(|s| !s.is_empty()).map(String::from);
            let mut image_2 = data.image_2.as_deref().filter(|s| !s.is_empty()).map(String::from);
            if image_1.is_none() && image_2.is_some() {
                image_1 = image_2.take();
            }

            let raw_json = serde_json::to_string(&raw).unwrap_or_default();
            let now = chrono::Local::now().timestamp();

            let _ = sqlx::query(INSERT_PRESET)
            .bind(data.id as i64)
            .bind(data.class as i64)
            .bind(&data.title)
            .bind(&data.user_nickname)
            .bind(&data.character_name)
            .bind(data.downloads as i64)
            .bind(data.views as i64)
            .bind(data.likes as i64)
            .bind(&image_2)
            .bind(&pab_filename)
            .bind(data.created_at)
            .bind(now)
            .bind(&raw_json)
            .execute(&pool)
            .await;

            let current = n_meta.fetch_add(1, Ordering::Relaxed) + 1;
            log.tag(&preset.class_hint, &format!("Meta ready: {}", preset.preset_id)).await;
            events::emit_progress(&app_h, ScrapperProgress {
                preset_id: preset.preset_id.to_string(),
                status: "metadata".to_string(),
                message: format!("Ready: {}", preset.preset_id),
                class_name: preset.class_hint.clone(),
                class_id: data.class,
                current,
                total,
            });

            let _ = tx.send(FetchedMeta { preset, data, preset_dir, image_1, image_2 }).await;
        });
    }
    drop(tx);

    let mut n_done = 0usize;
    let mut n_err = fetch_errors.load(Ordering::Relaxed);
    let mut img_current = 0usize;

    while let Some(meta) = rx.recv().await {
        img_current += 1;

        if cancel.load(Ordering::Relaxed) {
            log.tag("USER", "Sync cancelled").await;
            events::emit_progress(app, ScrapperProgress {
                preset_id: meta.preset.preset_id.to_string(),
                status: "cancelled".to_string(),
                message: "Cancelled".to_string(),
                class_name: meta.preset.class_hint.clone(),
                class_id: meta.data.class,
                current: img_current,
                total,
            });
            while rx.recv().await.is_some() {}
            break;
        }

        let (done, http_dl) = process_image(&meta, &browser, pool, popular_dir, &log, app, img_current, total).await;
        if done { n_done += 1; } else { n_err += 1; }
        if http_dl { image::jitter_sleep().await; }
    }

    events::emit_refresh_album(app);

    let msg = format!("Finished — {} done  {} error(s)", n_done, n_err);
    log.tag("SYNC", &msg).await;
    Ok(msg)
}

pub async fn watch_input_dir(app: AppHandle, input_dir: PathBuf) {
    use std::time::Duration;

    let pool = loop {
        if let Some(p) = app.state::<DbPool>().0.get() {
            break p.clone();
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    };

    let log = Logger::new(&pool, "scrapper_service");
    let mut known: HashSet<u64> = scan_input(&input_dir).into_iter().map(|p| p.preset_id).collect();
    log.tag("WATCH", &format!("Watching {} ({} known)", input_dir.display(), known.len())).await;

    loop {
        tokio::time::sleep(Duration::from_secs(20)).await;
        let current = scan_input(&input_dir);
        let new_files: Vec<String> = current.iter()
            .filter(|p| !known.contains(&p.preset_id))
            .filter_map(|p| p.pab_path.file_name().map(|n| n.to_string_lossy().into_owned()))
            .collect();
        known = current.into_iter().map(|p| p.preset_id).collect();
        if !new_files.is_empty() {
            log.tag("WATCH", &format!("New: {} preset(s)", new_files.len())).await;

            // Cancel any running operation so to_download gets priority
            let cancel = app.state::<ScrapperCancelToken>().0.clone();
            cancel.store(true, Ordering::Relaxed);
            tokio::time::sleep(Duration::from_millis(300)).await;
            cancel.store(false, Ordering::Relaxed);

            events::emit_folder_changed(&app, new_files);
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }
}
