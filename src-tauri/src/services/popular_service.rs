use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::AppHandle;

use crate::errors::AppError;
use crate::services::{events, garmoth_client::GarmothClient, playwright_service, scrapper_service::{ScrapperProgress, write_log, jitter_sleep}};

const DAYS_ALL: &[&str] = &["20", "30", "60", "90", "180", "365", "ever"];
const DAYS_PER_CLASS: &[&str] = &["180", "365", "ever"];

fn load_class_map(classes_dir: &Path) -> HashMap<u32, String> {
    let full_json = classes_dir.join("full.json");
    let Ok(content) = std::fs::read_to_string(&full_json) else { return HashMap::new() };
    let Ok(list) = serde_json::from_str::<Vec<serde_json::Value>>(&content) else { return HashMap::new() };
    list.into_iter().filter_map(|v| {
        let garmoth_id = v["garmoth_id"].as_u64()? as u32;
        let display = v["display"].as_str()?.to_string();
        Some((garmoth_id, display))
    }).collect()
}

fn find_image_in_presets(presets_dir: &Path, preset_id: u64, img_name: &str) -> Option<PathBuf> {
    let Ok(rd) = std::fs::read_dir(presets_dir) else { return None };
    for class_entry in rd.flatten() {
        let candidate = class_entry.path().join(preset_id.to_string()).join(img_name);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

pub async fn sync_popular(
    app: &AppHandle,
    popular_dir: &Path,
    presets_dir: &Path,
    classes_dir: &Path,
    logs_dir: &Path,
    cancel: Arc<AtomicBool>,
) -> Result<String, AppError> {
    write_log(logs_dir, "[POP  ] ─── Starting popular sync ───").await;

    let class_map = load_class_map(classes_dir);
    write_log(logs_dir, &format!("[POP  ] Class map loaded: {} classes", class_map.len())).await;
    if class_map.is_empty() {
        let msg = "Class map not found — run class init first";
        write_log(logs_dir, &format!("[ERR  ] {}", msg)).await;
        return Err(AppError::Scrape(msg.to_string()));
    }

    let client = Arc::new(GarmothClient::new(""));

    let mut seen: HashMap<u64, serde_json::Value> = HashMap::new();
    let mut total_raw = 0usize;

    // Phase 1: class=all for all 7 windows
    write_log(logs_dir, &format!("[POP  ] Phase 1 — class=all windows: {}", DAYS_ALL.join(", "))).await;
    let mut handles = Vec::new();
    for &days in DAYS_ALL {
        let client = Arc::clone(&client);
        let days_str = days.to_string();
        handles.push((days, tokio::spawn(async move {
            client.fetch_popular(None, &days_str).await
        })));
    }
    for (days, handle) in handles {
        match handle.await {
            Ok(Ok(items)) => {
                let n = items.len();
                total_raw += n;
                write_log(logs_dir, &format!("[POP  ] all/{} → {} results", days, n)).await;
                for item in items {
                    if let Some(id) = item["id"].as_u64() { seen.entry(id).or_insert(item); }
                }
            }
            Ok(Err(e)) => write_log(logs_dir, &format!("[ERR  ] all/{} failed: {}", days, e)).await,
            Err(e) => write_log(logs_dir, &format!("[ERR  ] all/{} panicked: {}", days, e)).await,
        }
    }
    write_log(logs_dir, &format!("[POP  ] Phase 1 done — raw: {}  unique so far: {}", total_raw, seen.len())).await;

    // Phase 2: per-class for the 3 longer windows
    write_log(logs_dir, &format!("[POP  ] Phase 2 — {} classes × {} windows", class_map.len(), DAYS_PER_CLASS.len())).await;
    let class_ids: Vec<(u32, String)> = class_map.iter().map(|(&id, name)| (id, name.clone())).collect();
    let mut handles2 = Vec::new();
    for (garmoth_id, class_name) in &class_ids {
        for &days in DAYS_PER_CLASS {
            let client = Arc::clone(&client);
            let days_str = days.to_string();
            let gid = *garmoth_id;
            let label = format!("{}/{}", class_name, days);
            handles2.push((label, tokio::spawn(async move {
                client.fetch_popular(Some(gid), &days_str).await
            })));
        }
    }
    let mut phase2_raw = 0usize;
    for (label, handle) in handles2 {
        match handle.await {
            Ok(Ok(items)) => {
                let n = items.len();
                phase2_raw += n;
                total_raw += n;
                if n > 0 {
                    write_log(logs_dir, &format!("[POP  ] {} → {} results", label, n)).await;
                }
                for item in items {
                    if let Some(id) = item["id"].as_u64() { seen.entry(id).or_insert(item); }
                }
            }
            Ok(Err(e)) => write_log(logs_dir, &format!("[ERR  ] {} failed: {}", label, e)).await,
            Err(e) => write_log(logs_dir, &format!("[ERR  ] {} panicked: {}", label, e)).await,
        }
    }
    write_log(logs_dir, &format!("[POP  ] Phase 2 done — raw: {}  unique after dedup: {}", phase2_raw, seen.len())).await;

    let unique: Vec<serde_json::Value> = seen.into_values().collect();
    let total = unique.len();
    write_log(logs_dir, &format!("[POP  ] Raw results: {}  Unique after dedup: {}", total_raw, total)).await;

    if total == 0 {
        write_log(logs_dir, "[POP  ] Nothing to sync — all windows returned empty").await;
        return Ok("No popular presets found".to_string());
    }

    // Count how many are already done
    let already_done = unique.iter().filter(|item| {
        let id = item["id"].as_u64().unwrap_or(0);
        let class_pa_id = item["class"].as_u64().unwrap_or(0) as u32;
        if let Some(display) = class_map.get(&class_pa_id) {
            popular_dir.join(display).join(id.to_string()).join(".ok").exists()
        } else {
            false
        }
    }).count();
    let to_process = total - already_done;
    write_log(logs_dir, &format!("[POP  ] Already synced: {}  To process: {}", already_done, to_process)).await;

    if to_process == 0 {
        write_log(logs_dir, "[POP  ] All presets already synced, nothing to do").await;
        return Ok(format!("Popular already up to date — {} presets", total));
    }

    write_log(logs_dir, "[POP  ] Starting browser session").await;
    let browser = match playwright_service::BrowserSession::new().await {
        Ok(s) => { write_log(logs_dir, "[POP  ] Browser session ready").await; s }
        Err(e) => {
            write_log(logs_dir, &format!("[ERR  ] Popular browser session failed: {}", e)).await;
            return Err(e);
        }
    };

    let mut n_done = 0usize;
    let mut n_skip = 0usize;
    let mut n_copy = 0usize;
    let mut n_download = 0usize;
    let mut n_err = 0usize;

    for (i, item) in unique.into_iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            write_log(logs_dir, "[USER ] Popular sync cancelled").await;
            break;
        }

        let id = match item["id"].as_u64() {
            Some(v) => v,
            None => {
                write_log(logs_dir, &format!("[WARN ] Item at index {} has no id — skipping", i)).await;
                continue;
            }
        };
        let class_pa_id = item["class"].as_u64().unwrap_or(0) as u32;
        let class_display = match class_map.get(&class_pa_id) {
            Some(d) => d.clone(),
            None => {
                write_log(logs_dir, &format!("[WARN ] Unknown class pa_id={} for preset {} — saved to Logs/Unknown", class_pa_id, id)).await;
                let unknown_dir = logs_dir.join("Unknown");
                let _ = tokio::fs::create_dir_all(&unknown_dir).await;
                let _ = tokio::fs::write(
                    unknown_dir.join(format!("{}.json", id)),
                    serde_json::to_string_pretty(&item).unwrap_or_default(),
                ).await;
                continue;
            }
        };

        let preset_dir = popular_dir.join(&class_display).join(id.to_string());

        if preset_dir.join(".ok").exists() {
            n_skip += 1;
            continue;
        }

        write_log(logs_dir, &format!("[POP  ] [{}/{}] Processing preset {} ({})", i + 1, total, id, class_display)).await;
        let _ = tokio::fs::create_dir_all(&preset_dir).await;

        let image_1 = item["image_1"].as_str().filter(|s| !s.is_empty()).map(String::from);
        let image_2 = item["image_2"].as_str().filter(|s| !s.is_empty()).map(String::from);
        let img_name = image_1.as_deref().or(image_2.as_deref());

        if image_1.is_none() && image_2.is_none() {
            write_log(logs_dir, &format!("[WARN ] Preset {} has no image_1 or image_2", id)).await;
        } else if image_1.is_none() {
            write_log(logs_dir, &format!("[POP  ] Preset {} has no image_1, using image_2 fallback", id)).await;
        }

        let mut json = item.clone();
        json["is_popular"] = serde_json::json!(true);
        json["images"] = serde_json::json!([]);
        json["updated_at"] = serde_json::json!(chrono::Local::now().timestamp());
        let _ = tokio::fs::write(
            preset_dir.join(format!("{}.json", id)),
            serde_json::to_string_pretty(&json).unwrap_or_default(),
        ).await;

        events::emit_popular_progress(app, ScrapperProgress {
            preset_id: id.to_string(),
            status: "metadata".to_string(),
            message: format!("Ready: {}", id),
            class_name: class_display.clone(),
            current: i + 1,
            total,
        });

        // Download or copy image
        let mut downloaded = false;
        if let Some(img) = img_name {
            let dest = preset_dir.join(img);
            let mut got_image = false;

            if let Some(src) = find_image_in_presets(presets_dir, id, img) {
                write_log(logs_dir, &format!("[POP  ] Found {} in Presets, copying", img)).await;
                if tokio::fs::copy(&src, &dest).await.is_ok() {
                    got_image = true;
                    n_copy += 1;
                    write_log(logs_dir, &format!("[POP  ] Copied image from Presets: preset {}", id)).await;
                } else {
                    write_log(logs_dir, &format!("[WARN ] Copy failed for preset {}, will download", id)).await;
                }
            }

            if !got_image {
                let url = format!(
                    "https://assets.garmoth.com/beauty-album/images/{}/{}",
                    class_pa_id, img
                );
                write_log(logs_dir, &format!("[POP  ] Downloading {} from assets", img)).await;
                match browser.download(&url).await {
                    Ok(bytes) => {
                        let _ = tokio::fs::write(&dest, &bytes).await;
                        got_image = true;
                        downloaded = true;
                        n_download += 1;
                        write_log(logs_dir, &format!("[POP  ] Downloaded image: preset {} ({}B)", id, bytes.len())).await;
                    }
                    Err(e) => {
                        n_err += 1;
                        write_log(logs_dir, &format!("[ERR  ] Image download failed for preset {}: {}", id, e)).await;
                    }
                }
            }

            if got_image {
                json["images"] = serde_json::json!([img]);
                let _ = tokio::fs::write(
                    preset_dir.join(format!("{}.json", id)),
                    serde_json::to_string_pretty(&json).unwrap_or_default(),
                ).await;
            }
        }

        let _ = tokio::fs::write(preset_dir.join(".ok"), b"").await;
        n_done += 1;
        write_log(logs_dir, &format!("[POP  ] Done preset {} [{}/{}]", id, n_done, to_process)).await;

        events::emit_popular_progress(app, ScrapperProgress {
            preset_id: id.to_string(),
            status: "done".to_string(),
            message: format!("Synced {}", id),
            class_name: class_display.clone(),
            current: i + 1,
            total,
        });

        if downloaded {
            jitter_sleep().await;
        }
    }

    let msg = format!(
        "Popular sync done — {} synced ({} copied, {} downloaded)  {} skipped  {} error(s)",
        n_done, n_copy, n_download, n_skip, n_err
    );
    write_log(logs_dir, &format!("[POP  ] {}", msg)).await;
    Ok(msg)
}
