use anyhow::Result;
use crate::config::AppConfig;

/// `shellm ask "查找所有大于10MB的文件"`
///
/// 流程：
/// 1. 构造 messages：system prompt（ask_system_prompt）+ user query
/// 2. 调用 ai::client::chat() 获取 AI 生成的命令
/// 3. 用 colored 高亮显示生成的命令
/// 4. 用 dialoguer 让用户确认是否执行
/// 5. 若确认，用 std::process::Command 执行并打印输出
pub async fn run(cfg: &AppConfig, query: &str) -> Result<()> {
    // TODO: 实现上述流程
    todo!("实现 ask 命令")
}
