use colored::Colorize;

/// 打印 AI 生成的命令（高亮）
pub fn print_command(cmd: &str) {
    // TODO: 美化输出
    println!("\n  {}\n", cmd.bold().green());
}

/// 打印解释内容
pub fn print_explanation(text: &str) {
    // TODO: 美化输出，可考虑 Markdown 渲染
    println!("{}", text);
}

/// 打印审查结果
pub fn print_review(text: &str) {
    // TODO: 美化输出
    println!("{}", text);
}

/// 打印错误信息
pub fn print_error(msg: &str) {
    eprintln!("{} {}", "error:".red().bold(), msg);
}
