
use crate::commands::execute;
use crate::config::AppConfig;
use anyhow::Result;
use rustyline::history::DefaultHistory;

/// `shellm shell`
pub async fn run(cfg: &AppConfig) -> Result<()> {
    println!("进入交互式 Shell 模式，输入 `exit` 退出。");
    let mut rl = rustyline::Editor::<(), DefaultHistory>::new()?;
    let mut session = crate::session::context::Session::new(20);
    loop {
        let readline = rl.readline("shellm> ");
        match readline {
            Ok(line) => {
                let cmd = line.trim();
                if cmd.is_empty() {
                    continue;
                }
                if cmd == "exit" || cmd == "quit" {
                    println!("👋 再见！");
                    break;
                }
                rl.add_history_entry(cmd)?;
                execute::run(cfg, cmd, Some(&mut session)).await?;
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                // Ctrl+C：取消当前输入，继续循环
                println!("^C");
                continue;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                // Ctrl+D：退出
                println!("👋 再见！");
                break;
            }
            Err(err) => {
                eprintln!("输入错误: {}", err);
                break;
            }
        }
    }
    Ok(())
}