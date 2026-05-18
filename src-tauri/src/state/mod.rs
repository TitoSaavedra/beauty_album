use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Child;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AppConfig {
    pub bdo_docs_dir: String,
}

impl AppConfig {
    pub fn presets_dir(&self) -> PathBuf {
        PathBuf::from(&self.bdo_docs_dir).join("Beauty Album").join("Presets")
    }

    pub fn customization_dir(&self) -> PathBuf {
        PathBuf::from(&self.bdo_docs_dir).join("Customization")
    }

    pub fn to_download_dir(&self) -> PathBuf {
        PathBuf::from(&self.bdo_docs_dir).join("Beauty Album").join("to_download")
    }

    pub fn logs_dir(&self) -> PathBuf {
        PathBuf::from(&self.bdo_docs_dir).join("Beauty Album").join("Logs")
    }
}

pub struct AppState(pub Mutex<AppConfig>);

pub struct ScrapperCancelToken(pub Arc<AtomicBool>);

impl Default for ScrapperCancelToken {
    fn default() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }
}

pub struct PythonProcess(pub Mutex<Option<Child>>);
