use anyhow::{anyhow, Context, Result};
use keyring::{Entry, Error as KeyringError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const SERVICE_NAME: &str = "convey";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub whisper_model: String,
    pub ai_model: String,
    pub language: Option<String>,
    pub auto_paste: bool,
    #[serde(default)]
    pub auto_paste_and_enter: bool,
    pub ai_processing_enabled: bool,
    pub system_prompt: Option<String>,
    pub hotkey: String,
    #[serde(default)]
    pub whisper_cli_path: Option<String>,
    #[serde(default)]
    pub recognize_press_enter: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            whisper_model: "whisper-1".to_string(),
            ai_model: "gpt-4o-mini".to_string(),
            language: Some("en".to_string()), // Default to English
            auto_paste: false, // Disabled by default due to accessibility permission requirements
            auto_paste_and_enter: false, // Paste and press Enter after transcription
            ai_processing_enabled: false, // Disabled by default for offline use
            system_prompt: Some(
                "You are a helpful assistant that cleans up and improves transcribed text. \
                 Fix grammar, punctuation, and formatting while preserving the original meaning."
                    .to_string(),
            ),
            hotkey: "Fn".to_string(), // Default to Fn key (Globe key on newer Macs)
            whisper_cli_path: None,
            recognize_press_enter: true, // Enable voice command "and press enter" detection by default
        }
    }
}

pub struct SecureStorage {
    config_path: PathBuf,
}

impl SecureStorage {
    pub fn new(config_path: PathBuf) -> Result<Self> {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(Self { config_path })
    }

    // Secure API key storage using system keychain
    pub fn store_api_key(&self, key_name: &str, api_key: &str) -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, key_name).context("Failed to create keyring entry")?;
        entry
            .set_password(api_key)
            .context("Failed to store API key in keychain")?;
        Ok(())
    }

    pub fn get_api_key(&self, key_name: &str) -> Result<String> {
        let entry = Entry::new(SERVICE_NAME, key_name).context("Failed to create keyring entry")?;
        match entry.get_password() {
            Ok(password) => Ok(password),
            Err(KeyringError::NoEntry) => Err(anyhow!("API key not found in keychain")),
            Err(err) => Err(anyhow!("Failed to retrieve API key from keychain: {}", err)),
        }
    }

    pub fn delete_api_key(&self, key_name: &str) -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, key_name).context("Failed to create keyring entry")?;
        match entry.delete_password() {
            Ok(_) | Err(KeyringError::NoEntry) => Ok(()),
            Err(err) => Err(anyhow!("Failed to delete API key from keychain: {}", err)),
        }?;
        Ok(())
    }

    pub fn has_api_key(&self, key_name: &str) -> Result<bool> {
        let entry = Entry::new(SERVICE_NAME, key_name).context("Failed to create keyring entry")?;
        match entry.get_password() {
            Ok(password) => Ok(!password.is_empty()),
            Err(KeyringError::NoEntry) => Ok(false),
            Err(err) => Err(anyhow!("Failed to check API key in keychain: {}", err)),
        }
    }

    // Settings storage in JSON file
    pub fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        let json =
            serde_json::to_string_pretty(settings).context("Failed to serialize settings")?;
        fs::write(&self.config_path, json).context("Failed to write settings file")?;
        Ok(())
    }

    pub fn load_settings(&self) -> Result<AppSettings> {
        if !self.config_path.exists() {
            let default_settings = AppSettings::default();
            self.save_settings(&default_settings)?;
            return Ok(default_settings);
        }

        let json = fs::read_to_string(&self.config_path).context("Failed to read settings file")?;
        let settings = serde_json::from_str(&json).context("Failed to deserialize settings")?;
        Ok(settings)
    }
}
