use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 应用配置
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    /// LLM API key
    pub api_key: Option<String>,
    /// api base url
    pub api_base: Option<String>,
    /// modal name
    pub model: Option<String>,
    /// language
    pub language: Option<String>,

}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_base: Some("https://api.openai.com".to_string()),
            model: Some("gpt-3.5-turbo".to_string()),
            language: Some("zh-CN".to_string()),
        }
    }
    
}

/// 获取配置文件路径：~/.config/shellm/config.json
pub fn config_path() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("com", "shellm", "shellm")
        .ok_or_else(|| anyhow::anyhow!("无法确定配置目录"))?;
    Ok(dirs.config_dir().join("config.json"))
}

/// 加载配置文件，不存在则返回默认值
pub fn load_config() -> Result<AppConfig> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    // TODO: 读取并反序列化配置文件
    todo!("实现配置文件加载")
}

/// 保存配置到文件
pub fn save_config(cfg: &AppConfig) -> Result<()> {
    let path = config_path()?;
    // TODO: 序列化并写入配置文件（确保目录存在）
    todo!("实现配置文件保存")
}
