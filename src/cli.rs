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
    /// 用自然语言描述你想做的事，AI 生成对应的 shell 命令
    Ask {
        /// 自然语言描述，例如 "查找当前目录下所有大于 10MB 的文件"
        query: String,
    },

    /// 解释一条 shell 命令的含义
    Explain {
        /// 要解释的命令，例如 "find . -name '*.rs' -mtime -7"
        command: String,
    },

    /// 让 AI 审查一个代码文件
    Review {
        /// 文件路径
        path: String,
    },

    /// 管理配置（API key 等）
    Config {
        /// 操作：set-key, show
        action: String,
    },
}
