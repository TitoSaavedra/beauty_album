use std::fs;
use std::path::Path;

use crate::errors::AppError;


pub fn get_classes(base_dir: &Path) -> Result<Vec<serde_json::Value>, AppError> {
    if !base_dir.exists() {
        return Ok(Vec::new());
    }

    let mut classes = Vec::new();

    for entry in fs::read_dir(base_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let icon_path = path.join("icon.svg");
        let icon = if icon_path.exists() {
            Some(icon_path.to_string_lossy().replace('\\', "/").to_string())
        } else {
            None
        };

        let preset_count = fs::read_dir(&path)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .filter(|e| {
                        let p = e.path();
                        if !p.is_dir() {
                            return false;
                        }
                        let id = p
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        p.join(format!("{}.json", id)).exists()
                    })
                    .count()
            })
            .unwrap_or(0);

        classes.push(serde_json::json!({
            "name": name,
            "icon_path": icon,
            "preset_count": preset_count
        }));
    }

    classes.sort_by(|a, b| {
        let ca = a["preset_count"].as_u64().unwrap_or(0);
        let cb = b["preset_count"].as_u64().unwrap_or(0);
        cb.cmp(&ca)
    });

    Ok(classes)
}

pub fn get_presets(base_dir: &Path, class_name: &str) -> Result<Vec<serde_json::Value>, AppError> {
    let class_dir = base_dir.join(class_name);
    if !class_dir.is_dir() {
        return Err(AppError::NotFound(format!("Class '{}' not found", class_name)));
    }

    let mut presets = Vec::new();

    for entry in fs::read_dir(&class_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let preset_id = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let json_path = path.join(format!("{}.json", preset_id));
        if !json_path.exists() {
            continue;
        }

        let content = match fs::read_to_string(&json_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let mut data: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let image_paths: Vec<String> = data
            .get("images")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|img| path.join(img).to_string_lossy().replace('\\', "/").to_string())
                    .collect()
            })
            .unwrap_or_default();

        data["image_paths"] = serde_json::json!(image_paths);
        data["preset_id"] = serde_json::json!(preset_id);

        let download_path = data
            .get("customization_file")
            .and_then(|v| v.as_str())
            .and_then(|f| {
                let p = path.join(f);
                if p.exists() {
                    Some(p.to_string_lossy().replace('\\', "/").to_string())
                } else {
                    None
                }
            });

        data["download_path"] = serde_json::json!(download_path);

        presets.push(data);
    }

    presets.sort_by(|a, b| {
        let da = a["downloads"].as_i64().unwrap_or(0);
        let db = b["downloads"].as_i64().unwrap_or(0);
        db.cmp(&da)
    });

    Ok(presets)
}
