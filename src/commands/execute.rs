use crate::config::AppConfig;
use crate::safety::ai_review;
use crate::safety::confirm;
use crate::safety::rules::{RiskLevel, RuleEngine};
use crate::session::context::Session;
use anyhow::Result;

pub async fn run(cfg: &AppConfig, query: &str, mut session: Option<&mut Session>) -> Result<()> {
    // 1. AI 生成命令
    let system_prompt = crate::ai::prompt::execute_system_prompt();
    let command = if let Some(session) = session.as_deref_mut() {
        if session.get_messages().is_empty() {
            session.add_message(crate::ai::client::Message::system(system_prompt));
        }
        // 无论是否首轮，都要把本轮 user query 加入 session
        session.add_message(crate::ai::client::Message::user(query.to_string()));
        let cmd = crate::ai::client::chat(cfg, session.get_messages()).await?;
        session.add_message(crate::ai::client::Message::assistant(cmd.clone()));
        cmd
    } else {
        let messages = [
            crate::ai::client::Message::system(system_prompt),
            crate::ai::client::Message::user(query.to_string()),
        ];
        crate::ai::client::chat(cfg, &messages).await?
    };
    println!("📋 生成命令: {}\n", command);

    // 2. 本地规则引擎快速评估
    let engine = RuleEngine::new();
    let (rule_level, rule_matches) = engine.evaluate_chain(&command);

    // 显示本地规则匹配结果
    if !rule_matches.is_empty() {
        let summary: String = rule_matches
            .iter()
            .map(|m| format!("- {} [{}] {}", m.level, m.rule_name, m.reason))
            .collect::<Vec<_>>()
            .join("\n");

        for m in &rule_matches {
            println!("  {} [{}] {}", m.level, m.rule_name, m.reason);
        }
        println!();

        if let Some(s) = session.as_deref_mut() {
            s.add_message(crate::ai::client::Message::user(format!(
                "本地规则扫描结果:\n{}",
                summary
            )));
        }
    }

    // 3. 中风险及以上触发 AI 深度审查
    let ai_result = if rule_level >= RiskLevel::Medium {
        let ctx = ai_review::gather_context();
        match ai_review::review(cfg, &command, Some(&ctx)).await {
            Ok(result) => Some(result),
            Err(e) => {
                eprintln!("  ⚠️  AI 安全审查失败，将仅依据本地规则: {}", e);
                if let Some(session) = session.as_deref_mut() {
                    session.add_message(crate::ai::client::Message::user(format!(
                        "AI 安全审查失败: {}",
                        e
                    )));
                }
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
            if let Some(s) = session.as_deref_mut() {
                s.add_message(crate::ai::client::Message::user(
                    "用户拒绝执行该命令".to_string(),
                ));
            }
            println!("  操作已取消。");
            return Ok(());
        }
        confirm::ConfirmResult::Blocked => {
            if let Some(s) = session.as_deref_mut() {
                s.add_message(crate::ai::client::Message::user(
                    "该命令被安全策略阻断".to_string(),
                ));
            }
            return Ok(());
        }
    }

    // 6. 通过安全审查后执行命令
    #[cfg(target_os = "windows")]
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "$OutputEncoding = [System.Text.Encoding]::UTF8; \
                 [Console]::OutputEncoding = [System.Text.Encoding]::UTF8; \
                 $ErrorActionPreference='SilentlyContinue'; & {{ {} }}",
                command
            ),
        ])
        .output()?;

    #[cfg(not(target_os = "windows"))]
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stdout.trim().is_empty() {
        println!("\n📊 命令输出：\n{}", stdout);
    }
    if !stderr.trim().is_empty() {
        println!("\n⚠️ 命令错误输出：\n{}", stderr);
    }

    // 摘要：stdout + stderr 各取前 30 行存入 session
    let stderr_summary: String = stderr.lines().take(20).collect::<Vec<_>>().join("\n");
    let stdout_summary: String = stdout.lines().take(30).collect::<Vec<_>>().join("\n");

    if output.status.success() {
        if let Some(session) = session.as_deref_mut() {
            session.add_message(crate::ai::client::Message::user(format!(
                "命令执行成功，输出摘要：\n{}",
                if stdout_summary.is_empty() {
                    "(无输出)"
                } else {
                    &stdout_summary
                }
            )));
        }
        println!("\n✅ 执行成功");
    } else {
        let exit_code = output.status.code().unwrap_or(-1);
        let session_msg = format!(
            "命令执行失败 (exit code: {})\n错误输出摘要：\n{}",
            exit_code,
            if stderr_summary.is_empty() {
                "(无错误输出)"
            } else {
                &stderr_summary
            }
        );
        if let Some(session) = session.as_deref_mut() {
            session.add_message(crate::ai::client::Message::user(session_msg));
        }
        println!("\n❌ 执行失败 (exit code: {})", exit_code);
    }
    Ok(())
}

pub fn run_raw(cmd: &str, mut session: Option<&mut Session>) -> Result<()> {
    let engine = RuleEngine::new();
    let (rule_level, rule_matches) = engine.evaluate_chain(&cmd);

    // 显示本地规则匹配结果
    if !rule_matches.is_empty() {
        let summary: String = rule_matches
            .iter()
            .map(|m| format!("- {} [{}] {}", m.level, m.rule_name, m.reason))
            .collect::<Vec<_>>()
            .join("\n");

        for m in &rule_matches {
            println!("  {} [{}] {}", m.level, m.rule_name, m.reason);
        }
        println!();

        if let Some(s) = session.as_deref_mut() {
            s.add_message(crate::ai::client::Message::user(format!(
                "本地规则扫描结果:\n{}",
                summary
            )));
        }
    }
    #[cfg(target_os = "windows")]
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "$OutputEncoding = [System.Text.Encoding]::UTF8; \
                 [Console]::OutputEncoding = [System.Text.Encoding]::UTF8; \
                 $ErrorActionPreference='SilentlyContinue'; & {{ {} }}",
                cmd
            ),
        ])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stdout.trim().is_empty() {
        println!("\n📊 命令输出：\n{}", stdout);
    }
    if !stderr.trim().is_empty() {
        println!("\n⚠️ 命令错误输出：\n{}", stderr);
    }

    // 摘要：stdout + stderr 各取前 30 行存入 session
    let stderr_summary: String = stderr.lines().take(20).collect::<Vec<_>>().join("\n");
    let stdout_summary: String = stdout.lines().take(30).collect::<Vec<_>>().join("\n");
    if output.status.success() {
        if let Some(session) = session.as_deref_mut() {
            session.add_message(crate::ai::client::Message::user(format!(
                "命令执行成功，输出摘要：\n{}",
                if stdout_summary.is_empty() {
                    "(无输出)"
                } else {
                    &stdout_summary
                }
            )));
        }
        println!("\n✅ 执行成功");
    } else {
        let exit_code = output.status.code().unwrap_or(-1);
        let session_msg = format!(
            "命令执行失败 (exit code: {})\n错误输出摘要：\n{}",
            exit_code,
            if stderr_summary.is_empty() {
                "(无错误输出)"
            } else {
                &stderr_summary
            }
        );
        if let Some(session) = session.as_deref_mut() {
            session.add_message(crate::ai::client::Message::user(session_msg));
        }
        println!("\n❌ 执行失败 (exit code: {})", exit_code);
    }
    Ok(())
}
