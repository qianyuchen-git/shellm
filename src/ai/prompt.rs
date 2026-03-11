/// Prompt templates for different commands

/// get current OS and shell info
fn system_info() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let shell = if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "Bash"
    };
    format!("The user is running on {os} ({arch}) with {shell}.")
}

/// execute command system prompt
pub fn execute_system_prompt() -> String {
    format!("You are a helpful assistant that generates shell commands based on user queries.
{system_info}
Generate commands that work on the user's operating system and shell.
When the user asks a question or describes a task, you should respond with a single shell command that accomplishes the task. Do not include any explanations or additional text, only the command itself.
Make sure the command is correct and efficient. If the user's request is ambiguous, choose a reasonable interpretation and generate a command accordingly.
Always respond in the same language as the user's input.", system_info = system_info())
}

/// explain command system prompt
pub fn explain_system_prompt() -> &'static str {
    // TODO: 编写 prompt
    todo!("编写 explain 命令的 system prompt")
}

/// review command system prompt
pub fn review_system_prompt() -> &'static str {
    // TODO: 编写 prompt
    todo!("编写 review 命令的 system prompt")
}
