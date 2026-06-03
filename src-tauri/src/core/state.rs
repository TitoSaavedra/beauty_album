use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AppConfig {
    pub bdo_docs_dir: String,
    #[serde(default)]
    pub cf_clearance: String,
}

impl AppConfig {
    fn root(&self) -> PathBuf {
        PathBuf::from(&self.bdo_docs_dir).join("Beauty Album")
    }

    /// DB folder: album.db lives here.
    pub fn db_dir(&self) -> PathBuf {
        self.root().join("DB")
    }

    pub fn db_path(&self) -> PathBuf {
        self.db_dir().join("album.db")
    }

    /// User preset files (.pab + images), organized as ClassName/id/.
    pub fn presets_dir(&self) -> PathBuf {
        self.root().join("Presets")
    }

    /// Popular preset images share the same Presets tree (ClassName/id/).
    pub fn popular_dir(&self) -> PathBuf {
        self.root().join("Presets")
    }

    /// Scrapper watches this folder for new .pab files.
    pub fn to_download_dir(&self) -> PathBuf {
        self.root().join("to_download")
    }

    /// BDO reads preset files from here — lives directly under bdo_docs_dir, not Beauty Album.
    pub fn customization_dir(&self) -> PathBuf {
        PathBuf::from(&self.bdo_docs_dir).join("Customization")
    }
}

pub struct AppState(pub Mutex<AppConfig>);

pub struct DbConn(pub OnceLock<DatabaseConnection>);

impl Default for DbConn {
    fn default() -> Self {
        Self(OnceLock::new())
    }
}
