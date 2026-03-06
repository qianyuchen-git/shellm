mod cli;
mod commands;
mod ai;
mod config;
mod output;
mod error;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 加载配置（API key 等）
    let cfg = config::load_config()?;

    match cli.command {
        Commands::Ask { query } => {
            commands::ask::run(&cfg, &query).await?;
        }
        Commands::Explain { command } => {
            commands::explain::run(&cfg, &command).await?;
        }
        Commands::Review { path } => {
            commands::review::run(&cfg, &path).await?;
        }
        Commands::Config { action } => {
            commands::config::run(&action)?;
        }
    }

    Ok(())
}
