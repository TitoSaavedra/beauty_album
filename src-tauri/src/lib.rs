mod commands;
mod errors;
mod services;
mod state;

use commands::album::{get_classes, get_presets, inject_preset, open_file};
use commands::config::{get_config, save_config};
use services::config_service;
use state::AppState;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let initial_config = config_service::load(app.handle());
            app.manage(AppState(Mutex::new(initial_config)));
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_classes,
            get_presets,
            get_config,
            save_config,
            inject_preset,
            open_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
