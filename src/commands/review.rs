use anyhow::Result;
use crate::config::AppConfig;

/// `shellm review src/main.rs`
///
/// 流程：
/// 1. 读取文件内容（注意校验文件是否存在）
/// 2. 构造 messages：system prompt（review_system_prompt）+ 文件内容
/// 3. 调用 ai::client::chat() 获取审查结果
/// 4. 格式化输出审查意见
pub async fn run(cfg: &AppConfig, path: &str) -> Result<()> {
    // TODO: 实现上述流程
    todo!("实现 review 命令")
}
