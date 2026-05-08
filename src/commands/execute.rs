use crate::config::AppConfig;
use crate::safety::ai_review;
use crate::safety::confirm;
use crate::safety::rules::{RiskLevel, RuleEngine};
use crate::session::context::Session;
use anyhow::Result;

/// 从 AI 原始回复中提取可执行的 shell 命令。
/// 考虑到弱模型偏常输出 markdown / 说明文字，这里做三层兑底：
/// 1) 如果被包在 ``` 代码块里，取代码块内容（跳过可能的语言标识行）
/// 2) 取第一个非空、非注释行，防止 AI 在命令后面加一大段说明
/// 3) 去掉行首可能的 shell prompt（`$ ` / `> `）
fn extract_command(raw: &str) -> String {
    let s = raw.trim();

    // 1) 剥离 markdown 代码块
    let body: &str = if let Some(rest) = s.strip_prefix("```") {
        if let Some(end) = rest.find("```") {
            &rest[..end]
        } else {
            rest
        }
    } else {
        s
    };

    // 2) 取第一个非空、非注释行
    let line = body
        .lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .unwrap_or("")
        .to_string();

    // 3) 去掉可能的 shell prompt 前缀
    line.trim_start_matches("$ ")
        .trim_start_matches("> ")
        .trim()
        .to_string()
}

pub async fn run(cfg: &AppConfig, query: &str, mut session: Option<&mut Session>) -> Result<()> {
    // 1. AI 生成命令
    let system_prompt = crate::ai::prompt::execute_system_prompt();
    let raw_command = if let Some(session) = session.as_deref_mut() {
        session.add_message(crate::ai::client::Message::user(query.to_string()));
        let cmd = crate::ai::client::chat(cfg, &session.get_messages()).await?;
        session.add_message(crate::ai::client::Message::assistant(cmd.clone()));
        cmd
    } else {
        let messages = [
            crate::ai::client::Message::system(system_prompt),
            crate::ai::client::Message::user(query.to_string()),
        ];
        crate::ai::client::chat(cfg, &messages).await?
    };
    let command = extract_command(&raw_command);
    if command.is_empty() {
        eprintln!("❌ AI 返回为空或未能提取出命令。原始回复:\n{}", raw_command);
        return Ok(());
    }
    println!("📋 生成命令: {}\n", command);

    // 2. 本地规则引擎快速评估
    let engine = RuleEngine::new();
    let (rule_level, rule_matches) = engine.evaluate_chain(&command);

    // 显示本地规则匹配结果
    if !rule_matches.is_empty() {
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
        ai_result
            .as_ref()
            .map(|r| r.level)
            .unwrap_or(RiskLevel::Low),
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
    let mut command_builder = std::process::Command::new("powershell");
    command_builder.args([
        "-NoProfile",
        "-Command",
        &format!(
            "$OutputEncoding = [System.Text.Encoding]::UTF8; \
             [Console]::OutputEncoding = [System.Text.Encoding]::UTF8; \
             $ErrorActionPreference='Continue'; & {{ {} }}; exit $LASTEXITCODE",
            command
        ),
    ]);
    if let Some(s) = session.as_deref_mut() {
        command_builder.current_dir(s.current_dir());
    }
    let output = command_builder.output()?;

    #[cfg(not(target_os = "windows"))]
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()?;

    if output.status.success() {
        println!("\n✅ 执行成功");
    } else {
        let exit_code = output.status.code().unwrap_or(-1);
        println!("\n❌ 执行失败 (exit code: {})", exit_code);
    }
    Ok(())
}

pub fn run_raw(cmd: &str, mut session: Option<&mut Session>) {
    let engine = RuleEngine::new();
    let (_rule_level, rule_matches) = engine.evaluate_chain(&cmd);

    // 显示本地规则匹配结果
    if !rule_matches.is_empty() {
        for m in &rule_matches {
            println!("  {} [{}] {}", m.level, m.rule_name, m.reason);
        }
        println!();
    }
    #[cfg(target_os = "windows")]
    let mut command_builder = std::process::Command::new("powershell");
    command_builder.args([
        "-NoProfile",
        "-Command",
        &format!(
            "$OutputEncoding = [System.Text.Encoding]::UTF8; \
             [Console]::OutputEncoding = [System.Text.Encoding]::UTF8; \
             $ErrorActionPreference='Continue'; & {{ {} }}; exit $LASTEXITCODE",
            cmd
        ),
    ]);
    if let Some(s) = session.as_deref_mut() {
        command_builder.current_dir(s.current_dir());
    }
    let output = match command_builder.output() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("❌ 无法启动命令 `{}`: {}", cmd, e);
            return;
        }
    };
    if output.status.success() {
        println!("\n✅ 执行成功");
    } else {
        let exit_code = output.status.code().unwrap_or(-1);
        println!("\n❌ 执行失败 (exit code: {})", exit_code);
    }
}
