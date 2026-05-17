mod commands;
mod errors;
mod services;
mod state;

use commands::album::{get_classes, get_presets, inject_preset, open_file, open_logs};
use commands::config::{get_config, save_config};
use commands::scrapper::{check_pending, run_scrapper, stop_scrapper};
use services::{config_service, scrapper_service};
use state::{AppState, PythonProcess, ScrapperCancelToken};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use tauri::Manager;

const PYTHON_SERVER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../src-python/main.py");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let initial_config = config_service::load(app.handle());
            app.manage(AppState(Mutex::new(initial_config)));
            app.manage(ScrapperCancelToken::default());

            let config_path = app
                .path()
                .app_config_dir()
                .map(|d| d.join("config.json"))
                .unwrap_or_default();

            #[cfg(debug_assertions)]
            let child = Command::new("python")
                .arg(PathBuf::from(PYTHON_SERVER))
                .arg(format!("--config-path={}", config_path.display()))
                .spawn()
                .ok();

            #[cfg(not(debug_assertions))]
            let child = {
                let exe = app.path().resource_dir()
                    .map(|d| d.join("server.exe"))
                    .unwrap_or_default();
                Command::new(&exe)
                    .arg(format!("--config-path={}", config_path.display()))
                    .spawn()
                    .ok()
            };

            app.manage(PythonProcess(Mutex::new(child)));

            let watch_handle = app.handle().clone();
            let (input_dir, album_dir) = {
                let state = app.state::<AppState>();
                let config = state.0.lock().unwrap();
                (
                    std::path::PathBuf::from(&config.album_input_dir),
                    std::path::PathBuf::from(&config.album_dir),
                )
            };
            tauri::async_runtime::spawn(async move {
                scrapper_service::watch_input_dir(watch_handle, input_dir, album_dir).await;
            });

            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_classes,
            get_presets,
            get_config,
            save_config,
            inject_preset,
            open_file,
            open_logs,
            run_scrapper,
            stop_scrapper,
            check_pending
        ])
        .build(tauri::generate_context!())
        .expect("error building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                if let Ok(mut guard) = app.state::<PythonProcess>().0.lock() {
                    if let Some(mut child) = guard.take() {
                        let _ = child.kill();
                    }
                }
            }
        });
}
