use std::path::{Path, PathBuf};

use super::browser::BrowserSession;
use crate::core::logger::Logger;

/// Returns the first path matching `{dir}/{preset_id}/{img_name}` across all search dirs.
pub fn find_image_in_dirs(search_dirs: &[&Path], preset_id: u64, img_name: &str) -> Option<PathBuf> {
    let id_str = preset_id.to_string();
    for &dir in search_dirs {
        let Ok(rd) = std::fs::read_dir(dir) else { continue };
        for entry in rd.flatten() {
            let candidate = entry.path().join(&id_str).join(img_name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Tries to copy from `search_dirs` first; falls back to HTTP download via browser.
/// Returns `true` if the image was obtained and written to `dest`.
pub async fn acquire_image(
    browser: &BrowserSession,
    search_dirs: &[&Path],
    preset_id: u64,
    class_id: u32,
    img_name: &str,
    dest: &Path,
    log: &Logger,
    tag: &str,
) -> bool {
    if let Some(src) = find_image_in_dirs(search_dirs, preset_id, img_name) {
        if tokio::fs::copy(&src, dest).await.is_ok() {
            log.tag(tag, &format!("Copied {}", img_name)).await;
            return true;
        }
    }

    let url = format!(
        "https://assets.garmoth.com/beauty-album/images/{}/{}",
        class_id, img_name
    );
    match browser.download(&url).await {
        Ok(bytes) => {
            let _ = tokio::fs::write(dest, &bytes).await;
            log.tag(tag, &format!("Downloaded {} ({}B)", img_name, bytes.len())).await;
            true
        }
        Err(e) => {
            log.tag(tag, &format!("ERR {} failed: {}", img_name, e)).await;
            false
        }
    }
}

pub async fn jitter_sleep() {
    use std::time::Duration;
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_millis();
    let delay_ms = 100u64 + (millis as u64 * 12 / 10);
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
}
