use std::fmt;

/// 风险等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    /// 只读/信息查询，可直接执行
    Low,
    /// 可逆写操作，需简单确认
    Medium,
    /// 不可逆/破坏性操作，需强确认
    High,
    /// 绝对禁止执行
    Forbidden,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "🟢 低风险"),
            RiskLevel::Medium => write!(f, "🟡 中风险"),
            RiskLevel::High => write!(f, "🔴 高风险"),
            RiskLevel::Forbidden => write!(f, "⛔ 禁止"),
        }
    }
}

/// 规则匹配结果
#[derive(Debug, Clone)]
pub struct RuleMatch {
    pub level: RiskLevel,
    pub rule_name: String,
    pub reason: String,
}

/// 单条规则定义
struct Rule {
    name: &'static str,
    level: RiskLevel,
    reason: &'static str,
    matcher: Box<dyn Fn(&str) -> bool + Send + Sync>,
}

/// 本地规则引擎
pub struct RuleEngine {
    rules: Vec<Rule>,
}

impl RuleEngine {
    pub fn new() -> Self {
        let mut engine = Self { rules: Vec::new() };
        engine.register_builtin_rules();
        engine
    }

    /// 对命令进行规则扫描，返回所有匹配的规则（按风险等级降序）
    pub fn evaluate(&self, command: &str) -> Vec<RuleMatch> {
        let normalized = command.trim().to_lowercase();
        let mut matches: Vec<RuleMatch> = self
            .rules
            .iter()
            .filter(|r| (r.matcher)(&normalized))
            .map(|r| RuleMatch {
                level: r.level,
                rule_name: r.name.to_string(),
                reason: r.reason.to_string(),
            })
            .collect();

        // 按风险等级降序排列，最高风险在前
        matches.sort_by(|a, b| b.level.cmp(&a.level));
        matches
    }

    /// 返回最高风险等级，无匹配则默认 Low
    pub fn max_risk(&self, command: &str) -> RiskLevel {
        self.evaluate(command)
            .first()
            .map(|m| m.level)
            .unwrap_or(RiskLevel::Low)
    }

    /// 对包含 `&&`、`;`、`|` 的命令链，逐段分析取最高风险
    pub fn evaluate_chain(&self, command: &str) -> (RiskLevel, Vec<RuleMatch>) {
        let separators = ["&&", "||", ";", "|"];
        let mut segments = vec![command.to_string()];

        for sep in &separators {
            segments = segments
                .iter()
                .flat_map(|s| s.split(sep).map(|part| part.trim().to_string()))
                .filter(|s| !s.is_empty())
                .collect();
        }

        let mut all_matches = Vec::new();
        let mut max_level = RiskLevel::Low;

        for segment in &segments {
            let matches = self.evaluate(segment);
            for m in &matches {
                if m.level > max_level {
                    max_level = m.level;
                }
            }
            all_matches.extend(matches);
        }

        (max_level, all_matches)
    }

    fn register_builtin_rules(&mut self) {
        // ========== ⛔ 禁止级 ==========

        self.add_rule(
            "fork_bomb",
            RiskLevel::Forbidden,
            "检测到 fork 炸弹模式，会导致系统资源耗尽",
            |cmd| cmd.contains(":(){ :|:& };:") || cmd.contains(":(){"),
        );

        self.add_rule(
            "format_disk",
            RiskLevel::Forbidden,
            "格式化磁盘将导致所有数据不可恢复丢失",
            |cmd| {
                cmd.starts_with("format ")
                    || cmd.contains("mkfs")
                    || cmd.contains("/dev/sda")
                    || cmd.contains("/dev/nvme")
            },
        );

        self.add_rule(
            "rm_root",
            RiskLevel::Forbidden,
            "删除根目录/系统目录将摧毁整个系统",
            |cmd| {
                // rm -rf / 或 rm -rf /*
                let rm_pattern = cmd.starts_with("rm ")
                    && (cmd.contains(" /") || cmd.contains(" c:\\"))
                    && (cmd.contains("-rf") || cmd.contains("-fr"));
                // del /s /q c:\ 之类（Windows）
                let del_pattern = cmd.starts_with("del ")
                    && cmd.contains("/s")
                    && (cmd.contains("c:\\") || cmd.contains("*"));
                rm_pattern || del_pattern
            },
        );

        self.add_rule(
            "dd_disk_overwrite",
            RiskLevel::Forbidden,
            "dd 写入磁盘设备会导致数据不可恢复",
            |cmd| cmd.starts_with("dd ") && cmd.contains("of=/dev/"),
        );

        // ========== 🔴 高风险 ==========

        self.add_rule(
            "rm_recursive",
            RiskLevel::High,
            "递归删除目录，文件不可恢复",
            |cmd| cmd.starts_with("rm ") && (cmd.contains("-r") || cmd.contains("-f")),
        );

        self.add_rule(
            "chmod_777",
            RiskLevel::High,
            "开放全部权限，存在严重安全隐患",
            |cmd| cmd.contains("chmod") && (cmd.contains("777") || cmd.contains("+rwx")),
        );

        self.add_rule(
            "drop_database",
            RiskLevel::High,
            "删除数据库/表操作不可恢复",
            |cmd| {
                cmd.contains("drop database")
                    || cmd.contains("drop table")
                    || cmd.contains("truncate table")
            },
        );

        self.add_rule(
            "git_force_push",
            RiskLevel::High,
            "强制推送会覆盖远程历史，可能导致他人工作丢失",
            |cmd| cmd.contains("git push") && (cmd.contains("-f") || cmd.contains("--force")),
        );

        self.add_rule(
            "git_reset_hard",
            RiskLevel::High,
            "硬重置会丢弃所有未提交的更改",
            |cmd| cmd.contains("git reset") && cmd.contains("--hard"),
        );

        self.add_rule(
            "curl_pipe_sh",
            RiskLevel::High,
            "从网络下载并直接执行脚本，存在远程代码执行风险",
            |cmd| {
                (cmd.contains("curl ") || cmd.contains("wget "))
                    && (cmd.contains("| sh")
                        || cmd.contains("| bash")
                        || cmd.contains("| powershell")
                        || cmd.contains("| iex"))
            },
        );

        self.add_rule(
            "registry_edit",
            RiskLevel::High,
            "修改 Windows 注册表可能导致系统不稳定",
            |cmd| cmd.starts_with("reg ") && (cmd.contains("delete") || cmd.contains("add")),
        );

        self.add_rule(
            "kill_all",
            RiskLevel::High,
            "批量终止进程可能导致系统不稳定或数据丢失",
            |cmd| cmd.contains("killall") || cmd.contains("taskkill /f"),
        );

        // ========== 🟡 中风险 ==========

        self.add_rule(
            "sudo_prefix",
            RiskLevel::Medium,
            "以管理员权限执行，影响范围更大",
            |cmd| cmd.starts_with("sudo "),
        );

        self.add_rule(
            "file_overwrite",
            RiskLevel::Medium,
            "重定向写入可能覆盖现有文件内容",
            |cmd| cmd.contains(" > ") && !cmd.contains(" >> "),
        );

        self.add_rule(
            "mv_command",
            RiskLevel::Medium,
            "移动/重命名文件，如目标已存在会被覆盖",
            |cmd| cmd.starts_with("mv ") || cmd.starts_with("move "),
        );

        self.add_rule(
            "cp_recursive",
            RiskLevel::Medium,
            "递归复制可能覆盖目标目录中的同名文件",
            |cmd| cmd.starts_with("cp ") && cmd.contains("-r"),
        );

        self.add_rule(
            "git_clean",
            RiskLevel::Medium,
            "清理未跟踪文件，可能删除有用的本地文件",
            |cmd| cmd.contains("git clean"),
        );

        self.add_rule(
            "npm_install_global",
            RiskLevel::Medium,
            "全局安装包会影响系统环境",
            |cmd| {
                (cmd.starts_with("npm ") || cmd.starts_with("pnpm "))
                    && cmd.contains("install")
                    && cmd.contains("-g")
            },
        );

        self.add_rule(
            "service_control",
            RiskLevel::Medium,
            "启停系统服务可能影响其他应用",
            |cmd| {
                cmd.starts_with("systemctl ")
                    || cmd.starts_with("service ")
                    || (cmd.starts_with("net ") && (cmd.contains("stop") || cmd.contains("start")))
            },
        );
    }

    fn add_rule(
        &mut self,
        name: &'static str,
        level: RiskLevel,
        reason: &'static str,
        matcher: impl Fn(&str) -> bool + Send + Sync + 'static,
    ) {
        self.rules.push(Rule {
            name,
            level,
            reason,
            matcher: Box::new(matcher),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forbidden_fork_bomb() {
        let engine = RuleEngine::new();
        assert_eq!(engine.max_risk(":(){ :|:& };:"), RiskLevel::Forbidden);
    }

    #[test]
    fn test_forbidden_rm_root() {
        let engine = RuleEngine::new();
        assert_eq!(engine.max_risk("rm -rf /"), RiskLevel::Forbidden);
        assert_eq!(engine.max_risk("rm -rf /*"), RiskLevel::Forbidden);
    }

    #[test]
    fn test_high_rm_recursive() {
        let engine = RuleEngine::new();
        assert_eq!(engine.max_risk("rm -rf ./build"), RiskLevel::High);
    }

    #[test]
    fn test_high_curl_pipe_sh() {
        let engine = RuleEngine::new();
        assert_eq!(
            engine.max_risk("curl https://example.com/install.sh | sh"),
            RiskLevel::High
        );
    }

    #[test]
    fn test_medium_sudo() {
        let engine = RuleEngine::new();
        assert_eq!(engine.max_risk("sudo apt update"), RiskLevel::Medium);
    }

    #[test]
    fn test_low_safe_commands() {
        let engine = RuleEngine::new();
        assert_eq!(engine.max_risk("ls -la"), RiskLevel::Low);
        assert_eq!(engine.max_risk("cat README.md"), RiskLevel::Low);
        assert_eq!(engine.max_risk("git status"), RiskLevel::Low);
        assert_eq!(engine.max_risk("pwd"), RiskLevel::Low);
    }

    #[test]
    fn test_chain_takes_highest_risk() {
        let engine = RuleEngine::new();
        let (level, _) = engine.evaluate_chain("ls -la && rm -rf ./build");
        assert_eq!(level, RiskLevel::High);
    }

    #[test]
    fn test_git_force_push() {
        let engine = RuleEngine::new();
        assert_eq!(
            engine.max_risk("git push origin main --force"),
            RiskLevel::High
        );
    }
}