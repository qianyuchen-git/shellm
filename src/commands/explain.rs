use anyhow::Result;
use crate::config::AppConfig;

/// `shellm explain "find . -name '*.rs' -mtime -7"`
pub async fn run(cfg: &AppConfig, command: &str) -> Result<()> {
    let system_prompt = crate::ai::prompt::explain_system_prompt();
    let messages = [
        crate::ai::client::Message::system(system_prompt),
        crate::ai::client::Message::user(command.to_string()),
    ];
    let explanation = crate::ai::client::chat(cfg, &messages).await?;
    println!("{}", explanation);
    Ok(())
}
