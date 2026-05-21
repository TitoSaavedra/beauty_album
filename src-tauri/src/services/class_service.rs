use std::collections::HashSet;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::errors::AppError;
use crate::services::events::{self, ClassInitProgress};
use crate::services::playwright_service::BrowserSession;
use crate::services::scrapper_service::write_log;

const GARMOTH_JS_URL: &str = "https://assets.garmoth.com/_static/_nuxt/DyMq0m4C.js";

#[derive(Serialize, Deserialize, Clone)]
pub struct ClassInfo {
    pub pa_id: u32,
    pub name: String,
    pub display: String,
    pub gender: String,
    pub damage_type: String,
    pub icon: String,
}

pub async fn init(classes_dir: &Path, logs_dir: &Path, app: &AppHandle, browser: &BrowserSession) -> Result<(), AppError> {
    let full_path = classes_dir.join("full.json");
    if full_path.exists() {
        write_log(logs_dir, "[CLASS] Already initialized, skipping").await;
        return Ok(());
    }

    write_log(logs_dir, "[CLASS] Starting — downloading Garmoth JS").await;
    let bytes = browser.download(GARMOTH_JS_URL).await?;
    let js = String::from_utf8_lossy(&bytes);

    let mut classes = parse_classes(&js);
    if classes.is_empty() {
        write_log(logs_dir, "[ERR  ] Class init: no classes parsed from JS").await;
        return Err(AppError::Scrape("no classes parsed from JS".to_string()));
    }

    let total = classes.len();
    write_log(logs_dir, &format!("[CLASS] Parsed {} classes", total)).await;
    tokio::fs::create_dir_all(classes_dir).await?;

    for (i, class) in classes.iter_mut().enumerate() {
        write_log(logs_dir, &format!("[CLASS] ({}/{}) {}", i + 1, total, class.display)).await;

        events::emit_class_init_progress(app, ClassInitProgress {
            current: i + 1,
            total,
            class_name: class.display.clone(),
        });

        let class_dir = classes_dir.join(&class.display);
        tokio::fs::create_dir_all(&class_dir).await?;

        let svg_url = format!("https://assets.garmoth.com/classes/svg/class_icon_{}.svg", class.pa_id);
        let icon_filename = format!("class_icon_{}.svg", class.pa_id);
        if let Ok(svg_bytes) = browser.download(&svg_url).await {
            tokio::fs::write(class_dir.join(&icon_filename), svg_bytes).await?;
            class.icon = icon_filename;
        } else {
            write_log(logs_dir, &format!("[WARN ] Icon download failed for {}", class.display)).await;
        }

        tokio::fs::write(
            class_dir.join(format!("{}.json", class.name)),
            serde_json::to_string_pretty(&*class)?,
        )
        .await?;
    }

    tokio::fs::write(&full_path, serde_json::to_string_pretty(&classes)?).await?;
    write_log(logs_dir, &format!("[CLASS] Done — {} classes saved", total)).await;

    Ok(())
}

fn parse_classes(js: &str) -> Vec<ClassInfo> {
    let marker = "pa_id:";
    let mut seen = HashSet::new();
    let mut classes = Vec::new();
    let mut pos = 0;

    while let Some(offset) = js[pos..].find(marker) {
        let start = pos + offset;
        let next = js[start + marker.len()..]
            .find(marker)
            .map(|p| start + marker.len() + p)
            .unwrap_or(js.len());
        let chunk = &js[start..next.min(start + 1500)];

        if let (Some(pa_id), Some(name), Some(display)) = (
            read_u32(chunk, "pa_id:"),
            read_str(chunk, "name:\""),
            read_str(chunk, "display:\""),
        ) {
            if seen.insert(pa_id) {
                let gender = read_str(chunk, "gender:\"").unwrap_or_default();
                let damage_type = read_str(chunk, "damageType:\"").unwrap_or_default();
                classes.push(ClassInfo {
                    pa_id,
                    name,
                    display,
                    gender,
                    damage_type,
                    icon: String::new(),
                });
            }
        }

        pos = start + marker.len();
    }

    classes.sort_by_key(|c| c.pa_id);
    classes
}

fn read_u32(s: &str, prefix: &str) -> Option<u32> {
    let start = s.find(prefix)? + prefix.len();
    let end = s[start..].find(|c: char| !c.is_ascii_digit())? + start;
    s[start..end].parse().ok()
}

fn read_str(s: &str, prefix: &str) -> Option<String> {
    let start = s.find(prefix)? + prefix.len();
    let end = s[start..].find('"')? + start;
    Some(s[start..end].to_string())
}
