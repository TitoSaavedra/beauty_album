use tauri::AppHandle;
use crate::core::events::{self, ScrapperProgress, ProgressType, ProgressStatus};

pub struct ProgressContext {
    app: AppHandle,
    preset_id: String,
    class_name: String,
    class_id: u32,
    current: usize,
    total: usize,
    progress_type: ProgressType,
}

impl ProgressContext {
    pub fn new_preset(
        app: &AppHandle,
        preset_id: String,
        class_name: String,
        class_id: u32,
        current: usize,
        total: usize,
    ) -> Self {
        Self {
            app: app.clone(),
            preset_id,
            class_name,
            class_id,
            current,
            total,
            progress_type: ProgressType::Preset,
        }
    }

    pub fn new_popular(
        app: &AppHandle,
        preset_id: String,
        class_name: String,
        class_id: u32,
        current: usize,
        total: usize,
    ) -> Self {
        Self {
            app: app.clone(),
            preset_id,
            class_name,
            class_id,
            current,
            total,
            progress_type: ProgressType::Popular,
        }
    }

    fn emit(&self, status: ProgressStatus, message: String) {
        let progress = ScrapperProgress {
            preset_id: self.preset_id.clone(),
            status,
            message,
            class_name: self.class_name.clone(),
            class_id: self.class_id,
            current: self.current,
            total: self.total,
            progress_type: self.progress_type.clone(),
        };
        events::emit_progress(&self.app, progress);
    }

    pub fn emit_processing(&self, message: String) {
        self.emit(ProgressStatus::Processing, message);
    }

    pub fn emit_done(&self, message: String) {
        self.emit(ProgressStatus::Done, message);
    }
}
