mod app;
mod beauty;
mod core;
mod db;

use app::commands::{get_config, save_config};
use beauty::commands::{
    check_pending, discard_preset, get_class_favorites, get_classes, get_popular_preset_by_id,
    get_popular_presets, get_popular_regions, get_popular_stats, get_presets, get_wanted,
    inject_preset, is_db_ready, open_file, open_url, run_scrapper, set_class_favorite,
    set_wanted, stop_scrapper, sync_popular, toggle_wanted,
};
use core::state::{AppState, DbConn, ScrapperCancelToken};
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let initial_config = app::service::load(app.handle());
            core::events::emit_config_loaded(app.handle(), &initial_config);
            app.manage(AppState(Mutex::new(initial_config)));
            app.manage(ScrapperCancelToken::default());
            app.manage(DbConn::default());

            let configured = {
                let state = app.state::<AppState>();
                let config = state.0.lock().unwrap();
                !config.bdo_docs_dir.is_empty()
            };

            if configured {
                let (db_path, input_dir) = {
                    let state = app.state::<AppState>();
                    let config = state.0.lock().unwrap();
                    for dir in &[
                        config.db_dir(),
                        config.presets_dir(),
                        config.to_download_dir(),
                        config.customization_dir(),
                    ] {
                        let _ = std::fs::create_dir_all(dir);
                    }
                    (config.db_path(), config.to_download_dir())
                };

                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    match core::db::open(&db_path, &app_handle).await {
                        Ok(conn) => {
                            let db_state = app_handle.state::<DbConn>();
                            let _ = db_state.0.set(conn);
                            core::events::emit_db_ready(&app_handle, true);
                        }
                        Err(e) => {
                            core::events::emit_db_ready(&app_handle, false);
                            eprintln!("Database failed: {}", e);
                        }
                    }
                    beauty::scraping::service::watch_input_dir(app_handle, input_dir).await;
                });
            }

            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            is_db_ready,
            get_classes,
            get_presets,
            get_config,
            save_config,
            inject_preset,
            open_file,
            open_url,
            run_scrapper,
            stop_scrapper,
            check_pending,
            sync_popular,
            get_popular_presets,
            get_popular_preset_by_id,
            get_popular_regions,
            get_popular_stats,
            discard_preset,
            get_wanted,
            set_wanted,
            toggle_wanted,
            get_class_favorites,
            set_class_favorite,
        ])
        .build(tauri::generate_context!())
        .expect("error building tauri application")
        .run(|_app, _event| {});
}
