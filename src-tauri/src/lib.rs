mod commands;
mod errors;
mod services;
mod state;

use commands::album::{get_classes, get_presets, init_classes, inject_preset, open_file, open_logs, open_url};
use commands::config::{get_config, save_config};
use commands::popular::{sync_popular, get_popular_classes, get_popular_presets, discard_preset, get_wanted, set_wanted};
use commands::scrapper::{check_pending, run_scrapper, stop_scrapper};
use services::{config_service, scrapper_service};
use state::{AppState, ScrapperCancelToken};
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let initial_config = config_service::load(app.handle());
            app.manage(AppState(Mutex::new(initial_config)));
            app.manage(ScrapperCancelToken::default());

            let configured = {
                let state = app.state::<AppState>();
                let config = state.0.lock().unwrap();
                !config.bdo_docs_dir.is_empty()
            };

            if configured {
                let (input_dir, logs_dir) = {
                    let state = app.state::<AppState>();
                    let config = state.0.lock().unwrap();
                    let dirs = vec![
                        config.presets_dir(),
                        config.classes_dir(),
                        config.to_download_dir(),
                        config.logs_dir(),
                        config.customization_dir(),
                        config.popular_dir(),
                    ];
                    for dir in dirs {
                        let _ = std::fs::create_dir_all(dir);
                    }
                    (config.to_download_dir(), config.logs_dir())
                };

                let watch_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    scrapper_service::watch_input_dir(watch_handle, input_dir, logs_dir).await;
                });
            }

            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_classes,
            get_presets,
            init_classes,
            get_config,
            save_config,
            inject_preset,
            open_file,
            open_url,
            open_logs,
            run_scrapper,
            stop_scrapper,
            check_pending,
            sync_popular,
            get_popular_classes,
            get_popular_presets,
            discard_preset,
            get_wanted,
            set_wanted
        ])
        .build(tauri::generate_context!())
        .expect("error building tauri application")
        .run(|_app, _event| {});
}
