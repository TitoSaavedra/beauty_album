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

            let configured = {
                let state = app.state::<AppState>();
                let config = state.0.lock().unwrap();
                !config.bdo_docs_dir.is_empty()
            };

            let child = if configured {
                let (album_dir, input_dir, logs_dir) = {
                    let state = app.state::<AppState>();
                    let config = state.0.lock().unwrap();
                    (config.presets_dir(), config.to_download_dir(), config.logs_dir())
                };

                #[cfg(debug_assertions)]
                let proc = Command::new("python")
                    .arg(PathBuf::from(PYTHON_SERVER))
                    .arg(format!("--album-dir={}", album_dir.display()))
                    .arg(format!("--input-dir={}", input_dir.display()))
                    .arg(format!("--log-dir={}", logs_dir.display()))
                    .spawn()
                    .ok();

                #[cfg(not(debug_assertions))]
                let proc = {
                    use std::os::windows::process::CommandExt;
                    const CREATE_NO_WINDOW: u32 = 0x08000000;
                    let exe = app.path().resource_dir()
                        .map(|d| d.join("server.exe"))
                        .unwrap_or_default();
                    Command::new(&exe)
                        .arg(format!("--album-dir={}", album_dir.display()))
                        .arg(format!("--input-dir={}", input_dir.display()))
                        .arg(format!("--log-dir={}", logs_dir.display()))
                        .creation_flags(CREATE_NO_WINDOW)
                        .spawn()
                        .ok()
                };

                let watch_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    scrapper_service::watch_input_dir(watch_handle, input_dir, logs_dir).await;
                });

                proc
            } else {
                None
            };

            app.manage(PythonProcess(Mutex::new(child)));

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
                        #[cfg(target_os = "windows")]
                        {
                            use std::os::windows::process::CommandExt;
                            let pid = child.id();
                            {
                                let _ = Command::new("taskkill")
                                    .creation_flags(0x08000000)
                                    .args(["/F", "/T", "/PID", &pid.to_string()])
                                    .spawn();
                            }
                        }
                        let _ = child.kill();
                    }
                }
            }
        });
}
