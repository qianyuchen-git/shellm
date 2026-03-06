use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

/// app config struct
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

/// get config file path
pub fn config_path() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("com", "shellm", "shellm")
        .ok_or_else(|| anyhow::anyhow!("Unable to determine config directory"))?;
    Ok(dirs.config_dir().join("config.json"))
}

/// load config from file, if not exist return default config
pub fn load_config() -> Result<AppConfig> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let content = fs::read_to_string(&path)?;
    let cfg: AppConfig = serde_json::from_str(&content)?;
    Ok(cfg)
}

/// save config to file
pub fn save_config(cfg: &AppConfig) -> Result<()> {
    let path = config_path()?;
    let json = serde_json::to_string_pretty(cfg)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, json)?;
    Ok(())
}
