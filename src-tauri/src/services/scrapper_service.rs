use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::errors::AppError;
use crate::services::{events, garmoth_client::{GarmothClient, GarmothPreset}, playwright_service};
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

struct FetchedMeta {
    preset: PendingPreset,
    data: GarmothPreset,
    raw: serde_json::Value,
    preset_dir: PathBuf,
    pab_filename: String,
    image_1: Option<String>,
    image_2: Option<String>,
}

fn build_metadata_json(data: &GarmothPreset, raw: &serde_json::Value, images: &[String], pab_filename: &str) -> serde_json::Value {
    serde_json::json!({
        "id": data.id,
        "title": data.title,
        "class": data.class,
        "creation_at": data.creation_at,
        "updated_at": chrono::Local::now().timestamp(),
        "image_1": data.image_1,
        "image_2": data.image_2,
        "user_nickname": data.user_nickname,
        "character_name": data.character_name,
        "downloads": data.downloads,
        "views": data.views,
        "likes": data.likes,
        "images": images,
        "customization_file": pab_filename,
        "_backup": raw,
    })
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
    let browser = match playwright_service::BrowserSession::new().await {
        Ok(s) => { write_log(logs_dir, &format!("{} Browser session ready", prefix)).await; s }
        Err(e) => {
            write_log(logs_dir, &format!("[ERR  ] Browser session failed: {}", e)).await;
            return Err(e);
        }
    };

    let mut fetched: Vec<FetchedMeta> = Vec::new();
    let mut n_done = 0usize;
    let mut n_err = 0usize;

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
            message: format!("Fetching {}", preset.preset_id),
            class_name: preset.class_hint.clone(),
            current: i + 1,
            total,
        });

        let (data, raw) = match client.fetch_preset(preset.preset_id).await {
            Ok(d) => d,
            Err(e) => {
                n_err += 1;
                write_log(logs_dir, &format!("[ERR  ] Meta {} failed: {}", preset.preset_id, e)).await;
                continue;
            }
        };

        let preset_dir = presets_dir
            .join(&preset.class_hint)
            .join(preset.preset_id.to_string());
        let _ = tokio::fs::create_dir_all(&preset_dir).await;

        let pab_filename = preset
            .pab_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let _ = tokio::fs::copy(&preset.pab_path, preset_dir.join(&pab_filename)).await;

        let image_1 = data.image_1.as_deref().filter(|s| !s.is_empty()).map(String::from);
        let image_2 = data.image_2.as_deref().filter(|s| !s.is_empty()).map(String::from);

        let skeleton = build_metadata_json(&data, &raw, &[], &pab_filename);
        let _ = tokio::fs::write(
            preset_dir.join(format!("{}.json", preset.preset_id)),
            serde_json::to_string_pretty(&skeleton).unwrap_or_default(),
        ).await;

        write_log(logs_dir, &format!("[META ] Preset {}", preset.preset_id)).await;
        events::emit_progress(app, ScrapperProgress {
            preset_id: preset.preset_id.to_string(),
            status: "metadata".to_string(),
            message: format!("Ready: {}", preset.preset_id),
            class_name: preset.class_hint.clone(),
            current: i + 1,
            total,
        });

        if let Some(ref img1) = image_1 {
            let url = format!(
                "https://assets.garmoth.com/beauty-album/images/{}/{}",
                data.class, img1
            );
            events::emit_progress(app, ScrapperProgress {
                preset_id: preset.preset_id.to_string(),
                status: "processing".to_string(),
                message: format!("Downloading {}", preset.preset_id),
                class_name: preset.class_hint.clone(),
                current: i + 1,
                total,
            });
            match browser.download(&url).await {
                Ok(bytes) => {
                    let _ = tokio::fs::write(preset_dir.join(img1), &bytes).await;
                    let updated = build_metadata_json(&data, &raw, &[img1.clone()], &pab_filename);
                    let _ = tokio::fs::write(
                        preset_dir.join(format!("{}.json", preset.preset_id)),
                        serde_json::to_string_pretty(&updated).unwrap_or_default(),
                    ).await;
                    let _ = tokio::fs::write(preset_dir.join(".ok"), b"").await;
                    n_done += 1;
                    write_log(logs_dir, &format!("{} Done: preset {}", prefix, preset.preset_id)).await;
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
                    n_err += 1;
                    write_log(logs_dir, &format!("[ERR  ] image_1 {} failed: {}", preset.preset_id, e)).await;
                }
            }
        } else {
                let _ = tokio::fs::write(preset_dir.join(".ok"), b"").await;
            n_done += 1;
            write_log(logs_dir, &format!("{} Done (no images): preset {}", prefix, preset.preset_id)).await;
            events::emit_progress(app, ScrapperProgress {
                preset_id: preset.preset_id.to_string(),
                status: "done".to_string(),
                message: format!("Synced preset {}", preset.preset_id),
                class_name: preset.class_hint.clone(),
                current: i + 1,
                total,
            });
        }

        fetched.push(FetchedMeta { preset, data, raw, preset_dir, pab_filename, image_1, image_2 });
        jitter_sleep().await;
    }

    events::emit_refresh_album(app);

    // image_2 pass
    let img2_list: Vec<&FetchedMeta> = fetched.iter()
        .filter(|m| m.image_2.is_some() && ok_exists(presets_dir, m.preset.preset_id))
        .collect();

    if !img2_list.is_empty() {
        write_log(logs_dir, &format!("{} Phase 2: downloading image_2 for {} preset(s)", prefix, img2_list.len())).await;
        for meta in img2_list {
            if cancel.load(Ordering::Relaxed) {
                write_log(logs_dir, "[USER ] Sync cancelled").await;
                break;
            }
            let img2 = meta.image_2.as_ref().unwrap();
            let url = format!(
                "https://assets.garmoth.com/beauty-album/images/{}/{}",
                meta.data.class, img2
            );
            match browser.download(&url).await {
                Ok(bytes) => {
                    let _ = tokio::fs::write(meta.preset_dir.join(img2), &bytes).await;
                    let images: Vec<String> = meta.image_1.iter().chain(std::iter::once(img2)).cloned().collect();
                    let updated = build_metadata_json(&meta.data, &meta.raw, &images, &meta.pab_filename);
                    let _ = tokio::fs::write(
                        meta.preset_dir.join(format!("{}.json", meta.preset.preset_id)),
                        serde_json::to_string_pretty(&updated).unwrap_or_default(),
                    ).await;
                    write_log(logs_dir, &format!("{} Done image_2: preset {}", prefix, meta.preset.preset_id)).await;
                }
                Err(e) => {
                    write_log(logs_dir, &format!("[ERR  ] image_2 {} failed: {}", meta.preset.preset_id, e)).await;
                }
            }
            jitter_sleep().await;
        }
    }

    let msg = format!("Finished — {} done  {} error(s)", n_done, n_err);
    write_log(logs_dir, &format!("{} {}", prefix, msg)).await;
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
