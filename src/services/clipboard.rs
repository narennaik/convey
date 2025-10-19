use std::sync::Mutex;

use anyhow::Result;
use arboard::Clipboard;

use crate::clipboard::ClipboardManager;

pub struct ClipboardService {
    manager: Mutex<ClipboardManager>,
}

impl ClipboardService {
    pub fn new(manager: ClipboardManager) -> Self {
        Self {
            manager: Mutex::new(manager),
        }
    }

    pub fn copy_text(&self, text: &str) -> Result<()> {
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(text)?;
        Ok(())
    }

    pub fn paste_text(&self, text: &str) -> Result<()> {
        self.manager
            .lock()
            .expect("clipboard manager poisoned")
            .paste_text(text)
    }

    pub fn paste_text_and_enter(&self, text: &str) -> Result<()> {
        self.manager
            .lock()
            .expect("clipboard manager poisoned")
            .paste_text_and_enter(text)
    }
}
