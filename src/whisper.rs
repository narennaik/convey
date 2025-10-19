use anyhow::{anyhow, Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use which::which;

#[derive(Debug, Serialize, Deserialize)]
pub struct WhisperConfig {
    pub model: String,
    pub language: Option<String>,
    pub cli_path: Option<String>,
}

pub struct WhisperClient {
    config: WhisperConfig,
}

impl WhisperClient {
    pub fn new(config: WhisperConfig) -> Self {
        Self { config }
    }

    pub async fn transcribe(&self, audio_path: &Path) -> Result<String> {
        // Run transcription using whisper-cli
        let audio_path = audio_path.to_path_buf();
        let language = self.config.language.clone();
        let cli_override = self.config.cli_path.clone();

        tokio::task::spawn_blocking(move || {
            Self::transcribe_with_cli(&audio_path, language.as_deref(), cli_override.as_deref())
        })
        .await
        .context("Failed to spawn blocking task")?
    }

    fn transcribe_with_cli(
        audio_path: &Path,
        language: Option<&str>,
        cli_override: Option<&str>,
    ) -> Result<String> {
        log::info!("transcribe_with_cli called for: {:?}", audio_path);

        let mut candidates = Vec::new();

        // Priority 1: Bundled resources (for production app)
        if let Ok(exe_path) = std::env::current_exe() {
            // On macOS, bundled resources are in .app/Contents/Resources
            if let Some(parent) = exe_path.parent() {
                let bundled_model = parent.join("../Resources/resources/models/ggml-base.bin");
                log::info!("Trying bundled model path: {:?}", bundled_model);
                if bundled_model.exists() {
                    candidates.push(bundled_model);
                }
            }
        }

        // Priority 2: Development path
        if let Ok(p) = std::env::current_dir() {
            let dev_model = p.join("resources/models/ggml-base.bin");
            log::info!("Trying dev model path: {:?}", dev_model);
            if dev_model.exists() {
                candidates.push(dev_model);
            }
        }

        // Priority 3: User data directory
        if let Some(dirs) = ProjectDirs::from("com", "narennaik", "Convey") {
            let data_model = dirs.data_dir().join("models/ggml-base.bin");
            log::info!("Trying model path: {:?}", data_model);
            if data_model.exists() {
                candidates.push(data_model);
            }
        }

        let model_path = candidates
            .into_iter()
            .next()
            .context(
                "Whisper model not found. The bundled model may be missing from the app package.",
            )?;

        log::info!("Using model path: {:?}", model_path);

        let cli_binary = resolve_whisper_cli(cli_override)?;
        log::info!("Resolved whisper-cli path: {:?}", cli_binary);

        // Build whisper-cli command
        let mut cmd = Command::new(&cli_binary);

        cmd.arg("-m").arg(&model_path).arg("-f").arg(audio_path);

        // Set language if specified
        if let Some(lang) = language {
            cmd.arg("-l").arg(lang);
            log::info!("Language set to: {}", lang);
        }

        // Output to text format
        cmd.arg("-otxt");

        log::info!("Executing whisper-cli command: {:?}", cmd);

        // Execute command
        let output = cmd.output().context("Failed to execute whisper-cli")?;

        log::info!("Whisper command completed with status: {}", output.status);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "whisper-cli failed with status {}: {}",
                output.status,
                stderr
            ));
        }

        // Read the output file
        let output_txt = audio_path.with_extension("wav.txt");
        log::info!("Looking for output file: {:?}", output_txt);

        if output_txt.exists() {
            log::info!("Output file exists, reading...");
            let transcription = std::fs::read_to_string(&output_txt)
                .context("Failed to read transcription output")?;

            log::info!("Transcription length: {} characters", transcription.len());

            // Clean up the output file
            let _ = std::fs::remove_file(output_txt);

            Ok(transcription.trim().to_string())
        } else {
            log::warn!("Output file not found, trying stdout");
            // If no file, try to parse stdout
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::info!("stdout: {}", stdout);
            log::info!("stderr: {}", stderr);
            Ok(stdout.trim().to_string())
        }
    }
}

fn resolve_whisper_cli(cli_override: Option<&str>) -> Result<PathBuf> {
    if let Some(value) = cli_override {
        let candidate = expand_home(value.trim());
        if candidate.exists() {
            return Ok(candidate);
        }
        if !value.contains(std::path::MAIN_SEPARATOR) && !value.contains('/') {
            if let Ok(found) = which(value) {
                return Ok(found);
            }
        }
        return Err(anyhow!(
            "Configured whisper-cli path not found: {}",
            candidate.display()
        ));
    }

    if let Ok(found) = which("whisper-cli") {
        return Ok(found);
    }

    let fallbacks = [
        "/opt/homebrew/bin/whisper-cli",
        "/usr/local/bin/whisper-cli",
        "/usr/bin/whisper-cli",
    ];

    for path in fallbacks {
        let candidate = PathBuf::from(path);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(anyhow!(
        "Unable to locate whisper-cli. Please install it or set the binary path in Settings."
    ))
}

fn expand_home(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(stripped);
        }
    } else if path == "~" {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home);
        }
    }
    PathBuf::from(path)
}
