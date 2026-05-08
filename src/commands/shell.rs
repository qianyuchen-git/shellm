use crate::ai::prompt;
use crate::commands::builtins::{BuiltinContext, BuiltinRegistry};
use crate::commands::execute;
use crate::config::AppConfig;
use crate::session::context::Session;
use anyhow::Result;
use rustyline::history::DefaultHistory;

enum Input {
    Empty,
    Exit,
    Builtin(String),
    Raw(String),
    Cd(String),
    Ai(String),
}

impl Input {
    fn parse(line: &str) -> Self {
        let cmd = line.trim();
        if cmd.is_empty() {
            Input::Empty
        } else if cmd == "exit" || cmd == "quit" {
            Input::Exit
        } else if let Some(builtin) = cmd.strip_prefix('/') {
            Input::Builtin(builtin.to_string())
        } else if let Some(raw) = cmd.strip_prefix('!') {
            Input::Raw(raw.to_string())
        } else if cmd == "cd" {
            Input::Cd(String::new())
        } else if let Some(arg) = cmd.strip_prefix("cd ") {
            Input::Cd(arg.to_string())
        } else {
            Input::Ai(cmd.to_string())
        }
    }
}

/// `shellm shell`
pub async fn run(cfg: &AppConfig) -> Result<()> {
    println!("进入交互式 Shell 模式，输入 `exit` 退出。");
    let mut rl = rustyline::Editor::<(), DefaultHistory>::new()?;
    let mut session = Session::new(5, prompt::execute_system_prompt());
    let registry = BuiltinRegistry::default();
    loop {
        let readline = rl.readline(&format!("shellm:{}> ", session.current_dir().display()));
        match readline {
            Ok(line) => {
                let cmd = line.trim();
                let input = Input::parse(cmd);
                if !matches!(input, Input::Empty) {
                    let _ = rl.add_history_entry(cmd);
                }
                match input {
                    Input::Empty => continue,
                    Input::Exit => {
                        println!("👋 再见！");
                        break;
                    }
                    Input::Builtin(builtin) => {
                        let mut ctx = BuiltinContext { session: &mut session };
                        registry.dispatch(&builtin, &mut ctx);
                    }
                    Input::Raw(raw) => execute::run_raw(&raw, Some(&mut session)),
                    Input::Cd(arg) => handle_cd(&arg, &mut session),
                    Input::Ai(query) => {
                        if let Err(e) = execute::run(cfg, &query, Some(&mut session)).await {
                            eprintln!("❌ AI 命令失败: {e:#}");
                        }
                    }
                }
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

fn handle_cd(arg: &str, session: &mut Session) {
    let arg = arg.trim();
    if arg.is_empty() {
        println!("❓ 用法: cd <目录路径>");
        return;
    }
    // join 的奇妙特性：如果 arg 是绝对路径，会直接替换；相对路径则拼接
    let target = session.current_dir().join(arg);
    match std::fs::canonicalize(&target) {
        Ok(p) if p.is_dir() => session.change_dir(p),
        Ok(p) => println!("❌ 不是目录: {}", p.display()),
        Err(e) => println!("❌ 无法进入 {}: {}", target.display(), e),
    }
}
