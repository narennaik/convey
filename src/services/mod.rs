pub mod clipboard;
pub mod history;
pub mod recorder;
pub mod settings;

use std::sync::Arc;

use clipboard::ClipboardService;
use history::HistoryService;
use recorder::RecorderService;
use settings::SettingsService;

/// Convenience container that holds all backend services.
#[derive(Clone)]
pub struct AppServices {
    pub recorder: Arc<RecorderService>,
    pub settings: Arc<SettingsService>,
    pub history: Arc<HistoryService>,
    pub clipboard: Arc<ClipboardService>,
}

impl AppServices {
    pub fn new(
        recorder: RecorderService,
        settings: SettingsService,
        history: HistoryService,
        clipboard: ClipboardService,
    ) -> Self {
        Self {
            recorder: Arc::new(recorder),
            settings: Arc::new(settings),
            history: Arc::new(history),
            clipboard: Arc::new(clipboard),
        }
    }
}
