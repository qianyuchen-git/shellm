use anyhow::{Ok, Result};
use crate::config::{ AppConfig, load_config, config_path, save_config };

use crate::cli::ConfigAction;

/// `shellm config set-key` 或 `shellm config show`
///
/// 流程：
/// - set-key: 交互式输入 API key，保存到配置文件
/// - show: 打印当前配置（API key 部分脱敏）
pub fn run(action: &ConfigAction) -> Result<()> {
    match action {
        ConfigAction::SetKey => {
            // TODO: 用 dialoguer 读取 API key，调用 config::save_config()
            todo!("实现 set-key")
        }
        ConfigAction::Show => {
            load_config().map(|cfg| {
                println!("当前配置:");
                println!("API Key: {}", "");
                println!("模型: {}", cfg.model.unwrap_or_default());
                println!("URL: {}", cfg.api_base.unwrap_or_default());
            })?;
            Ok(())
        }
        ConfigAction::SetModel { model } => {
            // TODO: 设置模型
            todo!("实现 set-model")
        }
        ConfigAction::SetUrl { url } => {
            // TODO: 设置 URL
            todo!("实现 set-url")
        }
        ConfigAction::Reset => {
            // TODO: 重置配置
            todo!("实现 reset")
        }
        _ => {
            anyhow::bail!("未知操作: {}，可用操作: set-key, show", action);
        }
    }
}
