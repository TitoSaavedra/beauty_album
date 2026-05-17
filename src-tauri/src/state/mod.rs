use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AppConfig {
    pub album_dir: String,
    pub bdo_output_dir: String,
}

pub struct AppState(pub Mutex<AppConfig>);
