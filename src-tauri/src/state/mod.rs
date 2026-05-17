use serde::{Deserialize, Serialize};
use std::process::Child;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AppConfig {
    pub album_dir: String,
    pub bdo_output_dir: String,
    pub album_input_dir: String,
}

pub struct AppState(pub Mutex<AppConfig>);

pub struct ScrapperCancelToken(pub Arc<AtomicBool>);

impl Default for ScrapperCancelToken {
    fn default() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }
}

pub struct PythonProcess(pub Mutex<Option<Child>>);
