use clap::{Parser, Subcommand};

/// shellm — AI-powered shell assistant
#[derive(Parser, Debug)]
#[command(name = "shellm", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Execute a shell command based on a natural language query
    Execute {
        /// natural language query, e.g. "find all .rs files modified in the last 7 days"
        query: String,
    },

    /// Explain a shell command
    Explain {
        /// command need to explain, e.g. "ls -la"
        command: String,
    },

    /// manage configuration (API key, model, etc.)
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    Shell
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    SetKey,
    SetModel { model: String },
    SetUrl { url: String },
    Show,
    Reset,
}
