use anyhow::Result;

/// `shellm config set-key` 或 `shellm config show`
///
/// 流程：
/// - set-key: 交互式输入 API key，保存到配置文件
/// - show: 打印当前配置（API key 部分脱敏）
pub fn run(action: &str) -> Result<()> {
    match action {
        "set-key" => {
            // TODO: 用 dialoguer 读取 API key，调用 config::save_config()
            todo!("实现 set-key")
        }
        "show" => {
            // TODO: 加载并打印配置
            todo!("实现 show")
        }
        _ => {
            anyhow::bail!("未知操作: {}，可用操作: set-key, show", action);
        }
    }
}
