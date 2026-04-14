use crate::config::AppConfig;
use crate::safety::ai_review;
use crate::safety::rules::{RiskLevel, RuleEngine};
use crate::safety::confirm;
use anyhow::Result;

pub async fn run(cfg: &AppConfig, query: &str) -> Result<()> {
    // 1. AI 生成命令
    let system_prompt = crate::ai::prompt::execute_system_prompt();
    let messages = [
        crate::ai::client::Message::system(system_prompt),
        crate::ai::client::Message::user(query.to_string()),
    ];
    let command = crate::ai::client::chat(cfg, &messages).await?;

    println!("📋 生成命令: {}\n", command);

    // 2. 本地规则引擎快速评估
    let engine = RuleEngine::new();
    let (rule_level, rule_matches) = engine.evaluate_chain(&command);

    // 显示本地规则匹配结果
    if !rule_matches.is_empty() {
        println!("  ── 本地规则扫描 ──");
        for m in &rule_matches {
            println!("  {} [{}] {}", m.level, m.rule_name, m.reason);
        }
        println!();
    }

    // 3. 中风险及以上触发 AI 深度审查
    let ai_result = if rule_level >= RiskLevel::Medium {
        let ctx = ai_review::gather_context();
        match ai_review::review(cfg, &command, Some(&ctx)).await {
            Ok(result) => Some(result),
            Err(e) => {
                eprintln!("  ⚠️  AI 安全审查失败，将仅依据本地规则: {}", e);
                None
            }
        }
    } else {
        None
    };

    // 4. 取本地规则和 AI 审查中的较高风险等级
    let final_level = std::cmp::max(
        rule_level,
        ai_result.as_ref().map(|r| r.level).unwrap_or(RiskLevel::Low),
    );

    // 5. 根据风险等级进行差异化确认
    match confirm::confirm(&command, final_level, ai_result.as_ref()) {
        confirm::ConfirmResult::Approved => {}
        confirm::ConfirmResult::Rejected => {
            println!("  操作已取消。");
            return Ok(());
        }
        confirm::ConfirmResult::Blocked => {
            return Ok(());
        }
    }

    // 6. 通过安全审查后执行命令
    #[cfg(target_os = "windows")]
    let status = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!("$ErrorActionPreference='SilentlyContinue'; & {{ {} }}", command),
        ])
        .status()?;

    #[cfg(not(target_os = "windows"))]
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(&format!("{} 2>/dev/null", command))
        .status()?;

    if status.success() {
        println!("\n✅ 执行成功");
    } else {
        println!("\n❌ 执行失败 (exit code: {})", status.code().unwrap_or(-1));
    }
    Ok(())
}
