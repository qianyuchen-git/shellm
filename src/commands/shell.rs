use crate::ai::client::Role;
use crate::commands::execute;
use crate::config::AppConfig;
use crate::session;
use anyhow::Result;
use rustyline::history::DefaultHistory;

/// `shellm shell`
pub async fn run(cfg: &AppConfig) -> Result<()> {
    println!("进入交互式 Shell 模式，输入 `exit` 退出。");
    let mut rl = rustyline::Editor::<(), DefaultHistory>::new()?;
    let mut session = crate::session::context::Session::new(20);
    loop {
        let readline = rl.readline(&format!("shellm:{}> ", session.current_dir().display()));
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
                if cmd.starts_with("/") {
                    // 内置命令
                    let builtin_cmd = cmd.trim_start_matches("/");
                    handle_builtin(builtin_cmd, &mut session);
                    continue;
                }
                if cmd.starts_with('!') {
                    let raw_cmd = cmd.trim_start_matches("!");
                    execute::run_raw(raw_cmd, Some(&mut session))?;
                    continue;
                }
                if cmd.starts_with("cd") {
                    let parts: Vec<_> = cmd.splitn(2, ' ').collect();
                    if parts.len() < 2 {
                        println!("❓ 用法: cd <目录路径>");
                        continue;
                    }
                    let target = session.current_dir().join(parts[1].trim());
                    match std::fs::canonicalize(&target) {
                        Ok(p) if p.is_dir() => session.change_dir(p),
                        Ok(p) => println!("❌ 不是目录: {}", p.display()),
                        Err(e) => println!("❌ 无法进入 {}: {}", target.display(), e),
                    }
                    continue;
                }
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

pub fn handle_builtin(cmd: &str, session: &mut session::context::Session) {
    match cmd {
        "history" => {
            let messages = session.get_messages();
            println!("历史记录:");
            let user_msgs: Vec<_> = messages
                .iter()
                .filter(|m| !matches!(m.role, Role::System))
                .collect();
            if user_msgs.is_empty() {
                println!("  (暂无对话)");
            }
            for (i, msg) in user_msgs.iter().enumerate() {
                println!("{}. [{}] {}", i + 1, msg.role, msg.content);
            }
        }
        "clear" => {
            session.clear();
            println!("历史已清除。");
        }
        "help" => {
            println!("📖 可用内置命令：");
            println!("  /help     显示本帮助信息");
            println!("  /history  查看当前会话的对话历史");
            println!("  /clear    清空当前会话上下文");
            println!("  exit / quit  退出 shell");
        }
        _ => {
            println!("❓ 未知命令: /{}，输入 /help 查看可用命令。", cmd);
        }
    }
}
