use std::io::{self, Write};

use crate::safety::rules::RiskLevel;
use crate::safety::ai_review::AiReviewResult;

/// 确认结果
#[derive(Debug, PartialEq)]
pub enum ConfirmResult {
    /// 用户同意执行
    Approved,
    /// 用户拒绝执行
    Rejected,
    /// 规则直接阻断，无需询问
    Blocked,
}

/// 根据风险等级和审查结果进行差异化确认
///
/// - Low: 直接放行
/// - Medium: 显示风险提示，y/n 确认
/// - High: 显示详细分析，要求输入命令关键词确认
/// - Forbidden: 直接阻断
pub fn confirm(
    command: &str,
    level: RiskLevel,
    ai_result: Option<&AiReviewResult>,
) -> ConfirmResult {
    match level {
        RiskLevel::Low => {
            println!("  {} 安全检查通过，直接执行。", level);
            ConfirmResult::Approved
        }
        RiskLevel::Medium => confirm_medium(command, ai_result),
        RiskLevel::High => confirm_high(command, ai_result),
        RiskLevel::Forbidden => confirm_forbidden(command, ai_result),
    }
}

/// 中风险：简单 y/n 确认
fn confirm_medium(command: &str, ai_result: Option<&AiReviewResult>) -> ConfirmResult {
    println!();
    println!("  ┌─────────────────────────────────────────┐");
    println!("  │           🟡 中风险操作警告              │");
    println!("  └─────────────────────────────────────────┘");
    println!();
    println!("  待执行命令: {}", command);

    if let Some(result) = ai_result {
        println!("  风险说明:   {}", result.reason);
        println!("  影响范围:   {}", result.impact);
        println!("  是否可逆:   {}", if result.reversible { "是" } else { "否" });
        if let Some(ref rollback) = result.rollback_suggestion {
            println!("  回滚建议:   {}", rollback);
        }
    }

    println!();
    ask_yes_no("  是否继续执行？(y/n): ")
}

/// 高风险：显示详细信息，要求输入命令首个关键词确认
fn confirm_high(command: &str, ai_result: Option<&AiReviewResult>) -> ConfirmResult {
    println!();
    println!("  ┌─────────────────────────────────────────┐");
    println!("  │         🔴 高风险操作 - 请谨慎确认       │");
    println!("  └─────────────────────────────────────────┘");
    println!();
    println!("  待执行命令: {}", command);

    if let Some(result) = ai_result {
        println!();
        println!("  ── AI 安全分析 ──────────────────────────");
        println!("  风险等级:   {}", result.level);
        println!("  风险说明:   {}", result.reason);
        println!("  影响范围:   {}", result.impact);
        println!("  是否可逆:   {}", if result.reversible { "✅ 是" } else { "❌ 否" });
        if let Some(ref rollback) = result.rollback_suggestion {
            println!("  回滚建议:   {}", rollback);
        }
        println!("  ──────────────────────────────────────────");
    }

    // 提取确认关键词：取命令的第一个词
    let confirm_keyword = extract_confirm_keyword(command);

    println!();
    println!("  ⚠️  此操作风险极高且可能不可逆！");
    println!("  如确认执行，请输入 \"{}\" 来确认：", confirm_keyword);
    print!("  > ");
    io::stdout().flush().unwrap();

    let input = read_line();
    if input.trim() == confirm_keyword {
        println!("  ✅ 已确认，准备执行...");
        ConfirmResult::Approved
    } else {
        println!("  ❌ 输入不匹配，已取消执行。");
        ConfirmResult::Rejected
    }
}

/// 禁止级：直接阻断，不给确认机会
fn confirm_forbidden(command: &str, ai_result: Option<&AiReviewResult>) -> ConfirmResult {
    println!();
    println!("  ┌─────────────────────────────────────────┐");
    println!("  │        ⛔ 危险操作已被阻断！             │");
    println!("  └─────────────────────────────────────────┘");
    println!();
    println!("  被阻断命令: {}", command);

    if let Some(result) = ai_result {
        println!("  阻断原因:   {}", result.reason);
        println!("  潜在影响:   {}", result.impact);
    }

    println!();
    println!("  该命令被安全策略禁止执行，无法跳过。");
    println!("  如确认需要执行，请手动在终端中操作。");

    ConfirmResult::Blocked
}

/// y/n 交互确认
fn ask_yes_no(prompt: &str) -> ConfirmResult {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let input = read_line();
    match input.trim().to_lowercase().as_str() {
        "y" | "yes" => {
            println!("  ✅ 已确认。");
            ConfirmResult::Approved
        }
        _ => {
            println!("  ❌ 已取消执行。");
            ConfirmResult::Rejected
        }
    }
}

/// 从命令中提取确认关键词（取第一个词）
fn extract_confirm_keyword(command: &str) -> &str {
    command
        .trim()
        .split_whitespace()
        .next()
        .unwrap_or(command.trim())
}

/// 读取一行用户输入
fn read_line() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("读取输入失败");
    input.trim().to_string()
}
