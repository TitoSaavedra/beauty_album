use tauri::{AppHandle, Manager, State};

use crate::core::db;
use crate::core::events;
use crate::core::state::{AppConfig, AppState, DbConn};
use crate::app::service as config_service;

#[tauri::command]
pub fn get_config(state: State<AppState>) -> Result<AppConfig, String> {
    let config = state.0.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
pub async fn save_config(
    config: AppConfig,
    app: AppHandle,
    state: State<'_, AppState>,
    db: State<'_, DbConn>,
) -> Result<(), String> {
    config_service::save(&app, &config).map_err(|e| e.to_string())?;
    *state.0.lock().map_err(|e| e.to_string())? = config.clone();
    events::emit_config_loaded(&app, &config);

    if !config.bdo_docs_dir.is_empty() && db.0.get().is_none() {
        let (db_path, _input_dir, _presets_dir, _popular_dir) = (
            config.db_path(),
            config.to_download_dir(),
            config.presets_dir(),
            config.popular_dir(),
        );
        for dir in &[
            config.db_dir(),
            config.presets_dir(),
            config.to_download_dir(),
            config.customization_dir(),
        ] {
            let _ = std::fs::create_dir_all(dir);
        }
        let app_handle = app;
        tauri::async_runtime::spawn(async move {
            match db::open(&db_path, &app_handle).await {
                Ok(conn) => {
                    let db_state = app_handle.state::<DbConn>();
                    let _ = db_state.0.set(conn);
                    events::emit_db_ready(&app_handle, true);
                }
                Err(e) => {
                    events::emit_db_ready(&app_handle, false);
                    eprintln!("Database failed: {}", e);
                }
            }
        });
    }

    Ok(())
}