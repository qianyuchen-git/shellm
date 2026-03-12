use crate::config::AppConfig;
use anyhow::Result;

pub async fn run(cfg: &AppConfig, query: &str) -> Result<()> {
    let system_prompt = crate::ai::prompt::execute_system_prompt();
    let messages = [
        crate::ai::client::Message::system(system_prompt),
        crate::ai::client::Message::user(query.to_string()),
    ];
    let command = crate::ai::client::chat(cfg, &messages).await?;

    println!("📋 Executing: {}\n", command);

    #[cfg(target_os = "windows")]
    let status = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!("$ErrorActionPreference='SilentlyContinue'; & {{ {} }}", command),
        ])
        .status()?;

    #[cfg(not(target_os = "windows"))]
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(&format!("{} 2>/dev/null", command))
        .status()?;

    if status.success() {
        println!("\n✅ executed successfully");
    } else {
        println!("\n❌ execution failed (exit code: {})", status.code().unwrap_or(-1));
    }
    Ok(())
}
