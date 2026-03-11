use crate::config::AppConfig;
use anyhow::Result;

pub async fn run(cfg: &AppConfig, query: &str) -> Result<()> {
    let system_prompt = crate::ai::prompt::execute_system_prompt();
    let messages = [
        crate::ai::client::Message::system(system_prompt),
        crate::ai::client::Message::user(query.to_string()),
    ];
    let command = crate::ai::client::chat(cfg, &messages).await?;
    // Windows 用 cmd，macOS/Linux 统一用 sh
    #[cfg(target_os = "windows")]
    let status = std::process::Command::new("cmd")
        .args(["/C", &command])
        .status()?;

    #[cfg(not(target_os = "windows"))] // ← 同时覆盖 macOS 和 Linux
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(&command)
        .status()?;

    println!("Command exited with status: {}", status);
    Ok(())
}
