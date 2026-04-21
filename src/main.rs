mod cli;
mod commands;
mod ai;
mod config;
mod output;
mod error;
mod safety;
mod session;
use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 加载配置（API key 等）
    let cfg = config::load_config()?;


    match cli.command {
        Commands::Execute { query } => {
            commands::execute::run(&cfg, &query, None).await?;
        }
        Commands::Explain { command } => {
            commands::explain::run(&cfg, &command).await?;
        }
        Commands::Config { action } => {
            commands::config::run(&action)?;
        }
        Commands::Shell => {
            commands::shell::run(&cfg).await?;
        }
    }

    Ok(())
}
