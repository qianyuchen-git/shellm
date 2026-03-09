use anyhow::{Ok, Result};
use crate::config::{ AppConfig, load_config, config_path, save_config };

use crate::cli::ConfigAction;

/// Execute config subcommands
/// - set-key: interactively input API key and save
/// - show: print current config (API key partially masked)
/// - set-model: set the model name
/// - set-url: set the API base URL
/// - reset: reset config to defaults
pub fn run(action: &ConfigAction) -> Result<()> {
    match action {
        ConfigAction::SetKey => {
            println!("Enter your API key:");
            let mut api_key = String::new();
            std::io::stdin().read_line(&mut api_key)?;
            let api_key = api_key.trim().to_string();
            let mut cfg = load_config()?;
            cfg.api_key = Some(api_key);
            save_config(&cfg)?;
            println!("API key saved successfully");
            Ok(())
        }
        ConfigAction::Show => {
            load_config().map(|cfg| {
                println!("Current config:");
                println!("API Key: {}", cfg.api_key.as_ref().map(|k| {
                    if k.len() <= 4 {
                        "*".repeat(k.len())
                    } else {
                        format!("{}{}", "*".repeat(k.len() - 4), &k[k.len() - 4..])
                    }
                }).unwrap_or_else(|| "Not set".to_string()));
                println!("Model: {}", cfg.model.as_deref().unwrap_or("Not set"));
                println!("URL: {}", cfg.api_base.as_deref().unwrap_or("Not set"));
                println!("Language: {}", cfg.language.as_deref().unwrap_or("Not set"));
            })?;
            Ok(())
        }
        ConfigAction::SetModel { model } => {
            let mut cfg = load_config()?;
            cfg.model = Some(model.clone());
            save_config(&cfg)?;
            println!("Model set to {}", model);
            Ok(())
        }
        ConfigAction::SetUrl { url } => {
            let mut cfg = load_config()?;
            cfg.api_base = Some(url.clone());
            save_config(&cfg)?;
            println!("API base URL set to {}", url);
            Ok(())
        }
        ConfigAction::Reset => {
            let cfg = AppConfig::default();
            save_config(&cfg)?;
            println!("Config reset to defaults");
            Ok(())
        }
        _ => {
            anyhow::bail!("Unknown action: {:?}，available actions: set-key, show, set-model, set-url, reset", action);
        }
    }
}
