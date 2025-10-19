use anyhow::Result;
use std::process::Command;
use std::thread;
use std::time::Duration;

pub struct ClipboardManager;

impl ClipboardManager {
    pub fn new() -> Self {
        Self
    }

    pub fn copy_text(&self, text: &str) -> Result<()> {
        let mut clipboard = arboard::Clipboard::new()?;
        clipboard.set_text(text)?;
        Ok(())
    }

    pub fn paste_text(&self, text: &str) -> Result<()> {
        // First, copy to clipboard using arboard
        let mut clipboard = arboard::Clipboard::new()?;
        clipboard.set_text(text)?;

        // Wait a bit for clipboard to update
        thread::sleep(Duration::from_millis(100));

        // Use AppleScript to simulate Cmd+V on macOS - much more stable than enigo
        #[cfg(target_os = "macos")]
        {
            let script = r#"
                tell application "System Events"
                    keystroke "v" using {command down}
                end tell
            "#;

            let output = Command::new("osascript")
                .arg("-e")
                .arg(script)
                .output();

            match output {
                Ok(result) => {
                    if result.status.success() {
                        Ok(())
                    } else {
                        let error_msg = String::from_utf8_lossy(&result.stderr);
                        Err(anyhow::anyhow!(
                            "Auto-paste failed: {}. The text has been copied to your clipboard - you can paste it manually with Cmd+V",
                            error_msg
                        ))
                    }
                }
                Err(e) => {
                    Err(anyhow::anyhow!(
                        "Failed to execute AppleScript for paste: {}. The text has been copied to your clipboard - you can paste it manually with Cmd+V",
                        e
                    ))
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            // For non-macOS platforms, just return success after copying to clipboard
            Ok(())
        }
    }

    pub fn paste_text_and_enter(&self, text: &str) -> Result<()> {
        // First, copy to clipboard using arboard
        let mut clipboard = arboard::Clipboard::new()?;
        clipboard.set_text(text)?;

        // Wait a bit for clipboard to update
        thread::sleep(Duration::from_millis(100));

        // Use AppleScript to simulate Cmd+V and then Enter on macOS
        #[cfg(target_os = "macos")]
        {
            let script = r#"
                tell application "System Events"
                    keystroke "v" using {command down}
                    delay 0.1
                    key code 36
                end tell
            "#;

            let output = Command::new("osascript")
                .arg("-e")
                .arg(script)
                .output();

            match output {
                Ok(result) => {
                    if result.status.success() {
                        Ok(())
                    } else {
                        let error_msg = String::from_utf8_lossy(&result.stderr);
                        Err(anyhow::anyhow!(
                            "Auto-paste-and-enter failed: {}. The text has been copied to your clipboard - you can paste it manually with Cmd+V",
                            error_msg
                        ))
                    }
                }
                Err(e) => {
                    Err(anyhow::anyhow!(
                        "Failed to execute AppleScript for paste-and-enter: {}. The text has been copied to your clipboard - you can paste it manually with Cmd+V",
                        e
                    ))
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            // For non-macOS platforms, just return success after copying to clipboard
            Ok(())
        }
    }
}
