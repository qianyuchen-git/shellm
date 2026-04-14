use anyhow::Result;
use serde::Deserialize;

use crate::config::AppConfig;
use crate::safety::rules::RiskLevel;
use crate::safety::rules;
use crate::safety::confirm;
use crate::ai::client::{chat, Message};

/// AI 安全审查的返回结果
#[derive(Debug, Clone)]
pub struct AiReviewResult {
    /// AI 判定的风险等级
    pub level: RiskLevel,
    /// 风险说明
    pub reason: String,
    /// 影响范围描述
    pub impact: String,
    /// 是否可逆
    pub reversible: bool,
    /// 回滚建议（如有）
    pub rollback_suggestion: Option<String>,
}

/// AI 返回的原始 JSON 结构
#[derive(Debug, Deserialize)]
struct AiRawResponse {
    risk_level: String,
    reason: String,
    impact: String,
    reversible: bool,
    rollback_suggestion: Option<String>,
}

/// 构造发给 AI 的安全审查 prompt
fn build_review_prompt(command: &str, context: Option<&str>) -> [Message; 1] {
    let ctx = context.unwrap_or("未提供额外上下文");
    [Message::system(format!(
        r#"你是一个命令行安全审查专家。请分析以下命令的安全风险。

待审查命令：`{command}`
执行上下文：{ctx}

请严格以 JSON 格式返回，不要包含任何其他内容：
{{
  "risk_level": "low | medium | high | forbidden",
  "reason": "风险原因的简�说明",
  "impact": "该命令可能造成的影响范围",
  "reversible": true/false,
  "rollback_suggestion": "回滚建议，无则为 null"
}}

判定标准：
- low: 只读操作、信息查询，无副作用
- medium: 有写操作但可逆，影响范围有限
- high: 不可逆操作、影响范围大、有数据丢失风险
- forbidden: 会导致系统崩溃、全盘数据丢失等灾难性后果"#
    ))]
}

/// 将 AI 返回的字符串风险等级转为枚举
fn parse_risk_level(level: &str) -> RiskLevel {
    match level.trim().to_lowercase().as_str() {
        "low" => RiskLevel::Low,
        "medium" => RiskLevel::Medium,
        "high" => RiskLevel::High,
        "forbidden" => RiskLevel::Forbidden,
        _ => RiskLevel::Medium, // 无法识别时保守处理
    }
}

/// 从 AI 原始响应文本中提取 JSON 部分
fn extract_json(text: &str) -> &str {
    // 处理 AI 可能用 ```json ... ``` 包裹的情况
    let trimmed = text.trim();
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            return &trimmed[start..=end];
        }
    }
    trimmed
}

/// 调用 AI 对命令进行安全审查
///
/// # Arguments
/// - `cfg`: 应用配置（包含 API key 等）
/// - `command`: 待审查的命令字符串
/// - `context`: 可选的执行上下文（如当前目录、操作系统等）
pub async fn review(
    cfg: &AppConfig,
    command: &str,
    context: Option<&str>,
) -> Result<AiReviewResult> {
    let prompt = build_review_prompt(command, context);

    // 复用已有的 AI 模块发送请求
    let raw_response = chat(cfg, &prompt).await?;

    // 解析 JSON 响应
    let json_str = extract_json(&raw_response);
    let parsed: AiRawResponse = serde_json::from_str(json_str).map_err(|e| {
        anyhow::anyhow!(
            "AI 安全审查返回格式异常，无法解析: {}\n原始响应: {}",
            e,
            raw_response
        )
    })?;

    Ok(AiReviewResult {
        level: parse_risk_level(&parsed.risk_level),
        reason: parsed.reason,
        impact: parsed.impact,
        reversible: parsed.reversible,
        rollback_suggestion: parsed.rollback_suggestion,
    })
}

/// 生成当前执行上下文信息（供 AI 参考）
pub fn gather_context() -> String {
    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "未知".to_string());

    let os = std::env::consts::OS;
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "未知".to_string());

    format!("操作系统: {os}, 当前目录: {cwd}, 当前用户: {user}")
}