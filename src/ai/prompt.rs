/// Prompt 模板集中管理
///
/// 思路：每个子命令有不同的 system prompt，引导 LLM 输出结构化结果

/// ask 命令的 system prompt
/// 引导 LLM：只输出可执行的 shell 命令，不要多余解释
pub fn ask_system_prompt() -> &'static str {
    // TODO: 编写 prompt
    todo!("编写 ask 命令的 system prompt")
}

/// explain 命令的 system prompt
/// 引导 LLM：逐步解释命令的每个部分
pub fn explain_system_prompt() -> &'static str {
    // TODO: 编写 prompt
    todo!("编写 explain 命令的 system prompt")
}

/// review 命令的 system prompt
/// 引导 LLM：从可读性、性能、安全性等角度审查代码
pub fn review_system_prompt() -> &'static str {
    // TODO: 编写 prompt
    todo!("编写 review 命令的 system prompt")
}
