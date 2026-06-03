mod app;
mod beauty;
mod core;
mod db;

use app::commands::{get_config, save_config};
use beauty::commands::{
    discard_preset, get_classes, get_preset_by_id,
    get_popular_presets, get_popular_regions, get_popular_stats, get_presets, get_wanted,
    inject_preset, is_db_ready, open_file, open_url,
    set_wanted, toggle_wanted, get_class_favorites, set_class_favorite,
};
use core::state::{AppState, DbConn};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let initial_config = app::service::load(app.handle());
            core::events::emit_config_loaded(app.handle(), &initial_config);
            app.manage(AppState(Mutex::new(initial_config.clone())));
            app.manage(DbConn::default());

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let configured = !initial_config.bdo_docs_dir.is_empty();

                if configured {
                    core::events::emit_progress(&app_handle, core::events::ScrapperProgress {
                        preset_id: "0".to_string(),
                        status: core::events::ProgressStatus::Processing,
                        message: "Initializing...".to_string(),
                        class_name: "INIT".to_string(),
                        class_id: 0,
                        current: 0,
                        total: 0,
                        progress_type: core::events::ProgressType::Preset,
                    });
                    let (db_path, input_dir, presets_dir, popular_dir) = {
                        for dir in &[
                            initial_config.db_dir(),
                            initial_config.presets_dir(),
                            initial_config.to_download_dir(),
                            initial_config.customization_dir(),
                        ] {
                            let _ = std::fs::create_dir_all(dir);
                        }
                        (
                            initial_config.db_path(),
                            initial_config.to_download_dir(),
                            initial_config.presets_dir(),
                            initial_config.popular_dir(),
                        )
                    };

                    let db_ok = match core::db::open(&db_path, &app_handle).await {
                        Ok(conn) => {
                            let db_state = app_handle.state::<DbConn>();
                            let _ = db_state.0.set(conn);
                            core::events::emit_db_ready(&app_handle, true);
                            true
                        }
                        Err(e) => {
                            core::events::emit_db_ready(&app_handle, false);
                            eprintln!("Database failed: {}", e);
                            false
                        }
                    };

                    if db_ok {
                        let app_h = app_handle.clone();
                        let db_state = app_handle.state::<DbConn>();
                        if let Some(db_h) = db_state.0.get() {
                            let cancel = Arc::new(AtomicBool::new(false));
                            let _ = beauty::scrapping::service::orchestrate_scraping(
                                &app_h,
                                &db_h,
                                &input_dir,
                                &presets_dir,
                                &popular_dir,
                                cancel,
                                false,
                            ).await;
                        }
                    }
                } else {
                    eprintln!("Not configured — skipping initialization");
                }
            });

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
            get_popular_presets,
            get_preset_by_id,
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
