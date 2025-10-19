use std::sync::Mutex;

use anyhow::Result;

use crate::storage::{AppSettings, SecureStorage};

pub struct SettingsService {
    storage: Mutex<SecureStorage>,
}

impl SettingsService {
    pub fn new(storage: SecureStorage) -> Self {
        Self {
            storage: Mutex::new(storage),
        }
    }

    pub fn load(&self) -> Result<AppSettings> {
        self.storage
            .lock()
            .expect("settings storage poisoned")
            .load_settings()
    }

    pub fn save(&self, settings: &AppSettings) -> Result<()> {
        self.storage
            .lock()
            .expect("settings storage poisoned")
            .save_settings(settings)
    }

    pub fn store_api_key(&self, key_name: &str, api_key: &str) -> Result<()> {
        self.storage
            .lock()
            .expect("settings storage poisoned")
            .store_api_key(key_name, api_key)
    }

    pub fn get_api_key(&self, key_name: &str) -> Result<String> {
        self.storage
            .lock()
            .expect("settings storage poisoned")
            .get_api_key(key_name)
    }

    pub fn delete_api_key(&self, key_name: &str) -> Result<()> {
        self.storage
            .lock()
            .expect("settings storage poisoned")
            .delete_api_key(key_name)
    }

    pub fn has_api_key(&self, key_name: &str) -> Result<bool> {
        self.storage
            .lock()
            .expect("settings storage poisoned")
            .has_api_key(key_name)
    }
}
