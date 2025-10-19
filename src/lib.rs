mod ai;
mod audio;
mod clipboard;
mod database;
mod notch;
mod services;
mod sound;
mod storage;
mod ui;
mod whisper;
mod workflow;

#[cfg(target_os = "macos")]
mod fn_key_monitor;

use audio::AudioRecorder;
use clipboard::ClipboardManager;
use database::Database;
use directories::ProjectDirs;
use services::{
    clipboard::ClipboardService, history::HistoryService, recorder::RecorderService,
    settings::SettingsService, AppServices,
};
use std::fs;
use storage::SecureStorage;

pub fn run() -> iced::Result {
    env_logger::init();

    let project_dirs = ProjectDirs::from("com", "narennaik", "Convey")
        .expect("Failed to resolve project directories");

    let config_path = project_dirs.config_dir().join("settings.json");
    let data_path = project_dirs.data_dir();
    if let Err(e) = fs::create_dir_all(project_dirs.config_dir()) {
        panic!("Failed to create config directory: {}", e);
    }
    if let Err(e) = fs::create_dir_all(project_dirs.data_dir()) {
        panic!("Failed to create data directory: {}", e);
    }
    let db_path = data_path.join("transcriptions.db");

    let storage = SecureStorage::new(config_path).expect("Failed to initialize storage");
    let database = Database::new(db_path).expect("Failed to initialize database");

    let services = AppServices::new(
        RecorderService::new(AudioRecorder::new()),
        SettingsService::new(storage),
        HistoryService::new(database),
        ClipboardService::new(ClipboardManager::new()),
    );

    ui::run(services)
}
