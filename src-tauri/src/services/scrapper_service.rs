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
    write_log(album_dir, "Scrapper started via FastAPI").await;

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
                    write_log(album_dir, "Scrapper cancelled by user").await;
                    break;
                }
            }
        }
    }

    write_log(album_dir, "Scrapper finished").await;
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

async fn write_log(album_dir: &Path, msg: &str) {
    use tokio::io::AsyncWriteExt;
    let path = album_dir.join("tauri.log");
    if let Ok(mut f) = tokio::fs::OpenOptions::new().create(true).append(true).open(&path).await {
        let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let line = format!("[{}] [INFO ] {}\n", ts, msg);
        let _ = f.write_all(line.as_bytes()).await;
    }
}
