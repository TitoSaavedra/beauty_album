use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::errors::AppError;
use crate::services::class_data::CLASSES;
use crate::services::events::{self, ClassInitProgress};
use crate::services::playwright_service::BrowserSession;
use crate::services::scrapper_service::write_log;

#[derive(Serialize, Deserialize, Clone)]
pub struct ClassInfo {
    pub pa_id: u32,
    pub garmoth_id: u32,
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

    let mut classes: Vec<ClassInfo> = CLASSES.iter().map(|&(garmoth_id, pa_id, class_name)| {
        ClassInfo {
            pa_id,
            garmoth_id,
            name: class_name.to_lowercase().replace(' ', "_"),
            display: class_name.to_string(),
            gender: String::new(),
            damage_type: String::new(),
            icon: String::new(),
        }
    }).collect();

    classes.sort_by_key(|c| c.pa_id);
    let total = classes.len();
    write_log(logs_dir, &format!("[CLASS] {} classes to initialize", total)).await;

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

        let icon_filename = format!("class_icon_{}.svg", class.pa_id);
        let icon_path = class_dir.join(&icon_filename);
        if icon_path.exists() {
            class.icon = icon_filename;
        } else {
            let svg_url = format!("https://assets.garmoth.com/classes/svg/class_icon_{}.svg", class.pa_id);
            if let Ok(svg_bytes) = browser.download(&svg_url).await {
                tokio::fs::write(&icon_path, svg_bytes).await?;
                class.icon = icon_filename;
            } else {
                write_log(logs_dir, &format!("[WARN ] Icon download failed for {}", class.display)).await;
            }
        }

        tokio::fs::write(
            class_dir.join(format!("{}.json", class.name)),
            serde_json::to_string_pretty(&*class)?,
        ).await?;
    }

    tokio::fs::write(&full_path, serde_json::to_string_pretty(&classes)?).await?;
    write_log(logs_dir, &format!("[CLASS] Done — {} classes saved", total)).await;

    Ok(())
}
