use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter};

use crate::errors::AppError;

const SERVER: &str = "http://127.0.0.1:8765";
const READY_POLL_MS: u64 = 200;
const READY_TIMEOUT_SECS: u64 = 30;

#[derive(Serialize, Clone)]
pub struct ScrapperProgress {
    pub preset_id: String,
    pub status: String,
    pub message: String,
    pub current: usize,
    pub total: usize,
}

pub async fn run(
    app: &AppHandle,
    album_dir: &Path,
    cancel: Arc<AtomicBool>,
) -> Result<(), AppError> {
    wait_for_server().await?;
    write_log(album_dir, "[SYNC ] Scrapper started via FastAPI").await;

    let client = reqwest::Client::new();

    client
        .post(format!("{}/scrape", SERVER))
        .send()
        .await
        .map_err(|e| AppError::Scrape(format!("POST /scrape failed: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::Scrape(format!("POST /scrape error: {}", e)))?;

    let response = client
        .get(format!("{}/events", SERVER))
        .send()
        .await
        .map_err(|e| AppError::Scrape(format!("GET /events failed: {}", e)))?;

    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;
    let mut buf = String::new();

    loop {
        tokio::select! {
            chunk = stream.next() => {
                match chunk {
                    Some(Ok(bytes)) => {
                        buf.push_str(&String::from_utf8_lossy(&bytes));
                        drain_events(&mut buf, app);
                    }
                    Some(Err(e)) => return Err(AppError::Scrape(format!("SSE read error: {}", e))),
                    None => break,
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                if cancel.load(Ordering::Relaxed) {
                    client.post(format!("{}/stop", SERVER)).send().await.ok();
                    write_log(album_dir, "[USER ] Scrapper cancelled by user").await;
                    break;
                }
            }
        }
    }

    write_log(album_dir, "[SYNC ] Scrapper finished").await;
    Ok(())
}

fn drain_events(buf: &mut String, app: &AppHandle) {
    loop {
        let Some(nl) = buf.find('\n') else { break };
        let line = buf[..nl].trim().to_string();
        buf.drain(..=nl);

        let data = line.strip_prefix("data:").map(str::trim).unwrap_or("");
        if data.is_empty() {
            continue;
        }
        let Ok(val) = serde_json::from_str::<Value>(data) else { continue };
        let msg_type = val["type"].as_str().unwrap_or("");

        if msg_type == "progress" {
            let _ = app.emit("scrapper_progress", ScrapperProgress {
                preset_id: val["preset_id"].as_str().unwrap_or("").to_string(),
                status:    val["status"].as_str().unwrap_or("").to_string(),
                message:   val["message"].as_str().unwrap_or("").to_string(),
                current:   val["current"].as_u64().unwrap_or(0) as usize,
                total:     val["total"].as_u64().unwrap_or(0) as usize,
            });
        } else if msg_type == "done" {
            let _ = app.emit("scrapper_progress", ScrapperProgress {
                preset_id: String::new(),
                status:    "done".into(),
                message:   format!(
                    "Finished — {} done  {} skipped  {} error(s)",
                    val["n_done"].as_u64().unwrap_or(0),
                    val["n_skip"].as_u64().unwrap_or(0),
                    val["n_error"].as_u64().unwrap_or(0),
                ),
                current: 0,
                total:   0,
            });
        }
    }
}

pub async fn watch_input_dir(
    app: AppHandle,
    input_dir: std::path::PathBuf,
    album_dir: std::path::PathBuf,
) {
    let mut known = collect_input_files(&input_dir);
    write_log(
        &album_dir,
        &format!("[WATCH] Watching input dir: {} ({} known file(s))", input_dir.display(), known.len()),
    ).await;

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
                &album_dir,
                &format!("[WATCH] New file(s) detected: {} — notifying frontend", new_files.join(", ")),
            ).await;
            let _ = app.emit("folder_changed", new_files);
            write_log(&album_dir, "[WATCH] Cooldown 60s before next watch poll").await;
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }
}

fn collect_input_files(dir: &std::path::Path) -> std::collections::HashSet<std::path::PathBuf> {
    let mut files = std::collections::HashSet::new();
    collect_recursive(dir, &mut files);
    files
}

fn collect_recursive(dir: &std::path::Path, files: &mut std::collections::HashSet<std::path::PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_file() {
            files.insert(p);
        } else if p.is_dir() {
            collect_recursive(&p, files);
        }
    }
}

pub async fn pending_count() -> Result<usize, AppError> {
    wait_for_server().await?;
    let client = reqwest::Client::new();
    let val = client
        .get(format!("{}/pending", SERVER))
        .send()
        .await
        .map_err(|e| AppError::Scrape(format!("GET /pending failed: {}", e)))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| AppError::Scrape(format!("parse /pending failed: {}", e)))?;
    Ok(val["count"].as_u64().unwrap_or(0) as usize)
}

async fn wait_for_server() -> Result<(), AppError> {
    let client = reqwest::Client::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(READY_TIMEOUT_SECS);
    loop {
        if client.get(format!("{}/health", SERVER)).send().await.is_ok() {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(AppError::Scrape(format!(
                "Python server not ready after {}s", READY_TIMEOUT_SECS
            )));
        }
        tokio::time::sleep(Duration::from_millis(READY_POLL_MS)).await;
    }
}

pub async fn write_log(album_dir: &Path, msg: &str) {
    use tokio::io::AsyncWriteExt;
    let path = album_dir.join("tauri.log");
    if let Ok(mut f) = tokio::fs::OpenOptions::new().create(true).append(true).open(&path).await {
        let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let line = format!("[{}] {}\n", ts, msg);
        let _ = f.write_all(line.as_bytes()).await;
    }
}

pub fn write_log_sync(album_dir: &Path, msg: &str) {
    use std::io::Write;
    let path = album_dir.join("tauri.log");
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let _ = writeln!(f, "[{}] {}", ts, msg);
    }
}
