use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::errors::AppError;
use crate::services::{events, garmoth_client::GarmothClient, playwright_service};
use crate::state::AppState;

#[derive(Serialize, Clone)]
pub struct ScrapperProgress {
    pub preset_id: String,
    pub status: String,
    pub message: String,
    pub class_name: String,
    pub current: usize,
    pub total: usize,
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

fn ok_exists(presets_dir: &Path, preset_id: u64) -> bool {
    let Ok(rd) = std::fs::read_dir(presets_dir) else { return false };
    rd.flatten().any(|class_entry| {
        class_entry
            .path()
            .join(preset_id.to_string())
            .join(".ok")
            .exists()
    })
}

pub fn pending_count(input_dir: &Path, presets_dir: &Path) -> usize {
    scan_input(input_dir)
        .iter()
        .filter(|p| !ok_exists(presets_dir, p.preset_id))
        .count()
}

pub async fn run(
    app: &AppHandle,
    input_dir: &Path,
    presets_dir: &Path,
    logs_dir: &Path,
    cancel: Arc<AtomicBool>,
    dev_mode: bool,
) -> Result<String, AppError> {
    let all = scan_input(input_dir);
    let pending: Vec<PendingPreset> = if dev_mode {
        all
    } else {
        all.into_iter().filter(|p| !ok_exists(presets_dir, p.preset_id)).collect()
    };

    let total = pending.len();
    let prefix = if dev_mode { "[DEV  ]" } else { "[SYNC ]" };
    write_log(logs_dir, &format!("{} Starting sync: {} preset(s) pending", prefix, total)).await;

    if total == 0 {
        return Ok("No presets pending".to_string());
    }

    let cf_clearance = {
        let state = app.state::<AppState>();
        let guard = state.0.lock().map_err(|e| AppError::Scrape(e.to_string()))?;
        let val = guard.cf_clearance.clone();
        drop(guard);
        val
    };

    let client = GarmothClient::new(&cf_clearance);

    write_log(logs_dir, &format!("{} Starting browser session", prefix)).await;
    let cf_browser = match playwright_service::BrowserSession::new().await {
        Ok(session) => session,
        Err(e) => {
            write_log(logs_dir, &format!("[ERR  ] Browser session failed: {}", e)).await;
            return Err(e);
        }
    };
    write_log(logs_dir, &format!("{} Browser session ready", prefix)).await;

    let mut n_done = 0usize;
    let mut n_error = 0usize;

    for (i, preset) in pending.into_iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            write_log(logs_dir, "[USER ] Sync cancelled").await;
            events::emit_progress(app, ScrapperProgress {
                preset_id: preset.preset_id.to_string(),
                status: "cancelled".to_string(),
                message: "Cancelled by user".to_string(),
                class_name: preset.class_hint.clone(),
                current: i + 1,
                total,
            });
            break;
        }

        events::emit_progress(app, ScrapperProgress {
            preset_id: preset.preset_id.to_string(),
            status: "processing".to_string(),
            message: format!("Fetching preset {}", preset.preset_id),
            class_name: preset.class_hint.clone(),
            current: i + 1,
            total,
        });

        let result = sync_preset(&client, &preset, presets_dir, &cf_browser).await;

        match result {
            Ok(()) => {
                n_done += 1;
                write_log(logs_dir, &format!("[SYNC ] Done: preset {}", preset.preset_id)).await;
                events::emit_progress(app, ScrapperProgress {
                    preset_id: preset.preset_id.to_string(),
                    status: "done".to_string(),
                    message: format!("Synced preset {}", preset.preset_id),
                    class_name: preset.class_hint.clone(),
                    current: i + 1,
                    total,
                });
            }
            Err(e) => {
                n_error += 1;
                write_log(logs_dir, &format!("[ERR  ] Preset {} failed: {}", preset.preset_id, e)).await;
                events::emit_progress(app, ScrapperProgress {
                    preset_id: preset.preset_id.to_string(),
                    status: "error".to_string(),
                    message: e.to_string(),
                    class_name: preset.class_hint.clone(),
                    current: i + 1,
                    total,
                });
                break;
            }
        }
        jitter_sleep().await;
    }

    let msg = format!("Finished — {} done  {} error(s)", n_done, n_error);
    write_log(logs_dir, &format!("[SYNC ] {}", msg)).await;
    Ok(msg)
}

async fn jitter_sleep() {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_millis(); // 0–999
    let delay_ms = 100u64 + (millis as u64 * 12 / 10); // 100–1298 ms
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
}

async fn sync_preset(
    client: &GarmothClient,
    pending: &PendingPreset,
    presets_dir: &Path,
    cf_browser: &playwright_service::BrowserSession,
) -> Result<(), AppError> {
    let data = client.fetch_preset(pending.preset_id).await?;

    let preset_dir = presets_dir
        .join(&pending.class_hint)
        .join(pending.preset_id.to_string());
    tokio::fs::create_dir_all(&preset_dir).await?;

    let img_names: Vec<&String> = [&data.image_1, &data.image_2]
        .iter()
        .filter_map(|o| o.as_ref())
        .filter(|s| !s.is_empty())
        .collect();

    let mut images: Vec<String> = Vec::new();
    for img_name in &img_names {
        let url = format!(
            "https://assets.garmoth.com/beauty-album/images/{}/{}",
            data.class, img_name
        );
        let bytes = cf_browser.download(&url).await?;
        tokio::fs::write(preset_dir.join(img_name.as_str()), &bytes).await?;
        images.push(img_name.to_string());
    }

    if images.is_empty() {
        return Err(AppError::Scrape("no images downloaded".to_string()));
    }

    let pab_filename = pending
        .pab_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    tokio::fs::copy(&pending.pab_path, preset_dir.join(&pab_filename)).await?;

    let metadata = serde_json::json!({
        "id": data.id,
        "title": data.title,
        "class": data.class,
        "creation_at": data.creation_at,
        "user_nickname": data.user_nickname,
        "character_name": data.character_name,
        "downloads": data.downloads,
        "views": data.views,
        "likes": data.likes,
        "images": images,
        "customization_file": pab_filename,
        "_backup": &data,
    });

    tokio::fs::write(
        preset_dir.join(format!("{}.json", pending.preset_id)),
        serde_json::to_string_pretty(&metadata)?,
    )
    .await?;

    tokio::fs::write(preset_dir.join(".ok"), b"").await?;

    Ok(())
}

pub async fn watch_input_dir(
    app: AppHandle,
    input_dir: PathBuf,
    logs_dir: PathBuf,
) {
    let mut known = collect_input_files(&input_dir);
    write_log(
        &logs_dir,
        &format!("[WATCH] Watching input dir: {} ({} known file(s))", input_dir.display(), known.len()),
    )
    .await;

    loop {
        tokio::time::sleep(Duration::from_secs(20)).await;
        let current = collect_input_files(&input_dir);
        let new_files: Vec<String> = current
            .difference(&known)
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .collect();
        known = current;
        if !new_files.is_empty() {
            write_log(
                &logs_dir,
                &format!("[WATCH] New file(s): {} — notifying frontend", new_files.join(", ")),
            )
            .await;
            events::emit_folder_changed(&app, new_files);
            write_log(&logs_dir, "[WATCH] Cooldown 60s").await;
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }
}

fn collect_input_files(dir: &Path) -> std::collections::HashSet<PathBuf> {
    let mut files = std::collections::HashSet::new();
    collect_recursive_files(dir, &mut files);
    files
}

fn collect_recursive_files(dir: &Path, files: &mut std::collections::HashSet<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_file() {
            files.insert(p);
        } else if p.is_dir() {
            collect_recursive_files(&p, files);
        }
    }
}

pub async fn write_log(logs_dir: &Path, msg: &str) {
    use tokio::io::AsyncWriteExt;
    let path = logs_dir.join("tauri.log");
    if let Ok(mut f) = tokio::fs::OpenOptions::new().create(true).append(true).open(&path).await {
        let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let line = format!("[{}] {}\n", ts, msg);
        let _ = f.write_all(line.as_bytes()).await;
    }
}

pub fn write_log_sync(logs_dir: &Path, msg: &str) {
    use std::io::Write;
    let path = logs_dir.join("tauri.log");
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let _ = writeln!(f, "[{}] {}", ts, msg);
    }
}
