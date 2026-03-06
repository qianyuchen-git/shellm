use anyhow::Result;
use crate::config::AppConfig;

/// `shellm explain "find . -name '*.rs' -mtime -7"`
///
/// 流程：
/// 1. 构造 messages：system prompt（explain_system_prompt）+ 用户传入的命令
/// 2. 调用 ai::client::chat() 获取解释文本
/// 3. 格式化输出（可以用 colored 分段高亮）
pub async fn run(cfg: &AppConfig, command: &str) -> Result<()> {
    // TODO: 实现上述流程
    todo!("实现 explain 命令")
}
