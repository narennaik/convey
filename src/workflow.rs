use crate::{
    ai::{AIClient, AIConfig},
    services::AppServices,
    storage::AppSettings,
    whisper::{WhisperClient, WhisperConfig},
};
use chrono::Utc;
use log::{error, info, warn};

/// Detects if the user said "and press enter" or similar at the end of the transcription
/// Returns (cleaned_text, should_press_enter)
fn detect_and_strip_enter_command(text: &str) -> (String, bool) {
    let patterns = [
        // Correct transcriptions - only full phrases with "and/then" + "enter"
        "and press enter",
        "and hit enter",
        "and press return",
        "and hit return",
        "then press enter",
        "then hit enter",
        // Common misrecognitions - but only with "and/then" prefix
        "and present enter",
        "and presence enter",
        "and pressing enter",
        "and president enter",
        "then present enter",
        "then pressing enter",
    ];

    for pattern in &patterns {
        // Check if the pattern appears at the end (with optional punctuation)
        let trimmed = text.trim_end_matches(&['.', '!', '?', ',', ';', ' '][..]);
        let trimmed_lower = trimmed.to_lowercase();

        if trimmed_lower.ends_with(pattern) {
            // Remove the pattern from the end
            let pattern_start = trimmed.len() - pattern.len();
            let cleaned = trimmed[..pattern_start].trim_end().to_string();
            info!("Detected enter command: '{}', cleaned text: '{}'", pattern, cleaned);
            return (cleaned, true);
        }
    }

    (text.to_string(), false)
}

pub async fn start_recording(services: AppServices) -> Result<(), String> {
    let temp_dir = std::env::temp_dir();
    let audio_path = temp_dir.join(format!("recording_{}.wav", Utc::now().timestamp()));

    crate::sound::play_start();
    services
        .recorder
        .start(audio_path)
        .map_err(|e| e.to_string())
}

pub async fn stop_recording_and_transcribe(services: AppServices) -> Result<String, String> {
    info!("Stop recording workflow started");

    let audio_path = services.recorder.stop().map_err(|e| {
        error!("Failed to stop recording: {}", e);
        e.to_string()
    })?;
    crate::sound::play_stop();
    info!("Recording stopped: {:?}", audio_path);

    let settings = services.settings.load().map_err(|e| {
        error!("Failed to load settings: {}", e);
        e.to_string()
    })?;

    let transcribed_text = transcribe_audio(&services, &settings, &audio_path).await?;

    // Check if the user said "and press enter" or similar phrases at the end (if enabled)
    let (final_text, should_press_enter) = if settings.recognize_press_enter {
        detect_and_strip_enter_command(&transcribed_text)
    } else {
        (transcribed_text.clone(), false)
    };

    if settings.auto_paste || settings.auto_paste_and_enter || !final_text.is_empty() {
        if let Err(e) = services.clipboard.copy_text(&final_text) {
            error!("Failed to copy text to clipboard: {}", e);
        } else {
            info!("Text copied to clipboard");
            // Press enter if:
            // 1. auto_paste_and_enter is enabled, OR
            // 2. user said "and press enter" AND auto_paste is enabled
            if settings.auto_paste_and_enter || (should_press_enter && settings.auto_paste) {
                if let Err(e) = services.clipboard.paste_text_and_enter(&final_text) {
                    warn!("Failed to auto-paste-and-enter (text is copied to clipboard): {}", e);
                } else {
                    info!("Text pasted and Enter pressed successfully");
                }
            } else if settings.auto_paste {
                if let Err(e) = services.clipboard.paste_text(&final_text) {
                    warn!("Failed to auto-paste (text is copied to clipboard): {}", e);
                } else {
                    info!("Text pasted successfully");
                }
            }
        }
    }

    let _ = std::fs::remove_file(&audio_path);
    info!("Workflow completed successfully");

    Ok(transcribed_text)
}

async fn transcribe_audio(
    services: &AppServices,
    settings: &AppSettings,
    audio_path: &std::path::Path,
) -> Result<String, String> {
    info!("Preparing Whisper transcription...");
    let whisper_config = WhisperConfig {
        model: settings.whisper_model.clone(),
        language: settings.language.clone(),
        cli_path: settings
            .whisper_cli_path
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
    };

    let whisper_client = WhisperClient::new(whisper_config);
    let mut transcribed_text = whisper_client.transcribe(audio_path).await.map_err(|e| {
        error!("Whisper transcription failed: {}", e);
        e.to_string()
    })?;
    info!("Transcription completed: {}", transcribed_text);

    let mut processed_text = None;

    if settings.ai_processing_enabled {
        info!("AI processing is enabled, retrieving API key...");
        let ai_key = services
            .settings
            .get_api_key("openai_api_key")
            .map_err(|e| {
                error!("Failed to get API key: {}", e);
                format!(
                    "Please set your OpenAI API key in settings for AI processing: {}",
                    e
                )
            })?;

        info!("Processing text with AI...");
        let ai_config = AIConfig {
            api_key: ai_key,
            model: settings.ai_model.clone(),
            system_prompt: settings.system_prompt.clone(),
        };

        let ai_client = AIClient::new(ai_config);
        let processed = ai_client
            .process_text(&transcribed_text)
            .await
            .map_err(|e| {
                error!("AI processing failed: {}", e);
                e.to_string()
            })?;

        processed_text = Some(processed.clone());
        transcribed_text = processed;
        info!("AI processing completed");
    }

    services
        .history
        .insert_transcription(
            &transcribed_text,
            processed_text.as_deref(),
            settings.language.as_deref(),
            None,
        )
        .map_err(|e| {
            error!("Failed to save to database: {}", e);
            e.to_string()
        })?;

    Ok(transcribed_text)
}
