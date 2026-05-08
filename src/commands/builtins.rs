//! 内置命令系统
//!
//! 通过 `BuiltinCommand` trait + `BuiltinRegistry` 注册表来分发 `/xxx` 风格的命令。
//! 新增命令只需：
//! 1. 定义一个 struct
//! 2. 为它 impl `BuiltinCommand`
//! 3. 在 `BuiltinRegistry::default()` 里 register 一下

use crate::ai::client::Role;
use crate::session::context::Session;
use anyhow::Result;
use std::collections::HashMap;

/// 内置命令执行时能拿到的上下文。
/// 目前只有 session，以后可能加 cfg、registry 自身等。
pub struct BuiltinContext<'a> {
    pub session: &'a mut Session,
}

/// 所有内置命令必须实现的 trait。
pub trait BuiltinCommand {
    /// 命令名（不含 `/`）
    fn name(&self) -> &'static str;
    /// 一行简短帮助
    fn help(&self) -> &'static str;
    /// 执行命令
    fn execute(&self, ctx: &mut BuiltinContext<'_>) -> Result<()>;
}

/// 注册表：按名字调度命令。
pub struct BuiltinRegistry {
    cmds: HashMap<&'static str, Box<dyn BuiltinCommand>>,
}

impl BuiltinRegistry {
    pub fn new() -> Self {
        Self { cmds: HashMap::new() }
    }

    pub fn register<C: BuiltinCommand + 'static>(&mut self, cmd: C) {
        self.cmds.insert(cmd.name(), Box::new(cmd));
    }

    /// 按名字派发；未知命令时打印提示。
    pub fn dispatch(&self, name: &str, ctx: &mut BuiltinContext<'_>) {
        match self.cmds.get(name) {
            Some(cmd) => {
                if let Err(e) = cmd.execute(ctx) {
                    eprintln!("❌ /{} 执行失败: {e:#}", name);
                }
            }
            None => {
                println!("❓ 未知命令: /{}，输入 /help 查看可用命令。", name);
            }
        }
    }

    /// 遍历所有已注册命令（按名字排序），用于 /help 自动生成。
    pub fn iter_sorted(&self) -> Vec<(&'static str, &'static str)> {
        let mut v: Vec<_> = self
            .cmds
            .values()
            .map(|c| (c.name(), c.help()))
            .collect();
        v.sort_by_key(|(n, _)| *n);
        v
    }
}

impl Default for BuiltinRegistry {
    /// 默认注册所有内置命令。
    fn default() -> Self {
        let mut reg = Self::new();
        reg.register(HistoryCommand);
        reg.register(ClearCommand);
        reg.register(HelpCommand);
        reg
    }
}

// ─────────────────────── 具体命令实现 ───────────────────────

pub struct HistoryCommand;

impl BuiltinCommand for HistoryCommand {
    fn name(&self) -> &'static str { "history" }
    fn help(&self) -> &'static str { "查看当前会话的对话历史" }

    fn execute(&self, ctx: &mut BuiltinContext<'_>) -> Result<()> {
        let messages = ctx.session.get_messages();
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
        Ok(())
    }
}

pub struct ClearCommand;

impl BuiltinCommand for ClearCommand {
    fn name(&self) -> &'static str { "clear" }
    fn help(&self) -> &'static str { "清空当前会话上下文" }

    fn execute(&self, ctx: &mut BuiltinContext<'_>) -> Result<()> {
        ctx.session.clear();
        println!("历史已清除。");
        Ok(())
    }
}

/// `/help` 命令：自身没法访问 registry，所以这里硬编码静态帮助。
/// 如果要做"自动从 registry 收集"，可以让 shell 层在分发前特判 help 并传入 registry。
pub struct HelpCommand;

impl BuiltinCommand for HelpCommand {
    fn name(&self) -> &'static str { "help" }
    fn help(&self) -> &'static str { "显示本帮助信息" }

    fn execute(&self, _ctx: &mut BuiltinContext<'_>) -> Result<()> {
        println!("📖 可用内置命令：");
        println!("  /help     显示本帮助信息");
        println!("  /history  查看当前会话的对话历史");
        println!("  /clear    清空当前会话上下文");
        println!("  exit / quit  退出 shell");
        Ok(())
    }
}
