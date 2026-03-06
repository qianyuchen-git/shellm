use anyhow::Result;
use crate::config::AppConfig;

/// 调用 LLM API 并返回文本响应
///
/// 思路：
/// 1. 从 config 中获取 api_key, api_base, model
/// 2. 用 reqwest 发送 POST 请求到 /v1/chat/completions（OpenAI 兼容接口）
/// 3. 解析 JSON 响应，提取 assistant message content
pub async fn chat(cfg: &AppConfig, messages: &[Message]) -> Result<String> {
    // TODO: 构造请求体，发送 HTTP 请求，解析响应
    todo!("实现 LLM API 调用")
}

/// 聊天消息
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
