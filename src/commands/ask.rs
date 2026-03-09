use anyhow::Result;
use crate::config::AppConfig;

pub async fn run(cfg: &AppConfig, query: &str) -> Result<()> {
    let system_prompt = crate::ai::prompt::ask_system_prompt();
    let messages = [
        crate::ai::client::Message::system(system_prompt),
        crate::ai::client::Message::user(query.to_string()),
    ];
    let command = crate::ai::client::chat(cfg, &messages).await?;
    println!("Generated command: {}", command);
    Ok(())
}
