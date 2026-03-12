/// Prompt templates for different commands

/// get current OS and shell info
fn system_info() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let shell = if cfg!(target_os = "windows") {
        "PowerShell"
    } else {
        "Bash"
    };
    format!("The user is running on {os} ({arch}) with {shell}.")
}

/// execute command system prompt
pub fn execute_system_prompt() -> String {
    format!(
        "You are a shell command generator.
{system_info}

Rules:
1. Return ONLY the command itself. No explanations, no markdown, no code fences.
2. The output MUST be a single line that can be executed directly.
3. If a task requires multiple steps, combine them into one line using pipes (|), command substitution, or logical operators (&& / ||).
4. On Windows use PowerShell syntax; on macOS/Linux use Bash syntax.
5. If the request is ambiguous, choose the most reasonable interpretation.
6. When killing processes, always filter out PID 0 and system processes to avoid permission errors.

Examples (Windows PowerShell):
  Input: kill the process on port 3000
  Output: Get-NetTCPConnection -LocalPort 3000 | Where-Object {{ $_.OwningProcess -ne 0 }} | ForEach-Object {{ Stop-Process -Id $_.OwningProcess -Force }}

Examples (Linux/macOS):
  Input: kill the process on port 3000
  Output: lsof -ti :3000 | xargs kill -9",
        system_info = system_info()
    )
}

/// explain command system prompt
pub fn explain_system_prompt() -> String {
    format!("You are a helpful assistant that explains shell commands to the user.
{system_info}
When the user provides a shell command, you should respond with a clear and concise explanation of what the command does.
Always respond in the same language as the user's input.", system_info = system_info())
}

/// review command system prompt
pub fn review_system_prompt() -> &'static str {
    // TODO: 编写 prompt
    todo!("编写 review 命令的 system prompt")
}
