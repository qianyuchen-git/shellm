use anyhow::Result;
use crate::{ai::client, config::AppConfig};
use reqwest::Client;

/// post messages to AI and get response
pub async fn chat(cfg: &AppConfig, messages: &[Message]) -> Result<String> {
    let client = Client::new();
    let request_body = serde_json::json!({
        "model": cfg.model.as_deref().unwrap_or("gpt-3.5-turbo"),
        "messages": messages.iter().map(|m| {
            let role = match m.role {
                Role::System => "system",
                Role::User => "user",
            };
            serde_json::json!({
                "role": role,
                "content": m.content,
            })
        }).collect::<Vec<_>>(),
    });
    let response = client.post(format!("{}/v1/chat/completions", cfg.api_base.as_deref().unwrap_or("https://api.openai.com")))
        .bearer_auth(cfg.api_key.as_deref().ok_or_else(|| anyhow::anyhow!("API key not set"))?)
        .json(&request_body)
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;
    let content = response["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid response format: missing content"))?;
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
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: Role::System, content: content.into() }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self { role: Role::User, content: content.into() }
    }
}
