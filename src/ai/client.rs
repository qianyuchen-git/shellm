use anyhow::Result;
use crate::{config::AppConfig};
use reqwest::Client;
use crate::error::{AiError};

/// post messages to AI and get response
pub async fn chat(cfg: &AppConfig, messages: &[Message]) -> Result<String, AiError> {
    let client = Client::new();
    let request_body = serde_json::json!({
        "model": cfg.model.as_deref().unwrap_or("gpt-3.5-turbo"),
        "messages": messages.iter().map(|m| {
            let role = match m.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
            };
            serde_json::json!({
                "role": role,
                "content": m.content,
            })
        }).collect::<Vec<_>>(),
    });
    let response = client.post(format!("{}/v1/chat/completions", cfg.api_base.as_deref().unwrap_or("https://api.openai.com")))
        .bearer_auth(cfg.api_key.as_deref().ok_or_else(|| AiError::MissingApiKey)?)
        .json(&request_body)
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;
    let content = response["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| AiError::InvalidResponse {
            reason: "missing content".to_string(),
            raw: response.to_string(),
        })?;
    Ok(content.to_string())
}

/// Chat message
#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone)]
pub enum Role {
    System,
    User,
    Assistant,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let role_str = match self {
            Role::System => "System",
            Role::User => "User",
            Role::Assistant => "Assistant",
        };
        write!(f, "{}", role_str)
    }
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: Role::System, content: content.into() }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self { role: Role::User, content: content.into() }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: Role::Assistant, content: content.into() }
    }
}
