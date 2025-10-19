use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AIConfig {
    pub api_key: String,
    pub model: String,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

pub struct AIClient {
    client: reqwest::Client,
    config: AIConfig,
}

impl AIClient {
    pub fn new(config: AIConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    pub async fn process_text(&self, text: &str) -> Result<String> {
        let system_prompt = self.config.system_prompt.as_deref()
            .unwrap_or("You are a helpful assistant that cleans up and improves transcribed text. Fix grammar, punctuation, and formatting while preserving the original meaning.");

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: text.to_string(),
            },
        ];

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages,
            temperature: 0.3,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.config.api_key)
            .json(&request)
            .send()
            .await
            .context("Failed to send AI processing request")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse AI response")?;

        let processed_text = chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .context("No response from AI")?;

        Ok(processed_text)
    }
}
