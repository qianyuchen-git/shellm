use thiserror::Error;

/// 应用级错误类型
#[derive(Error, Debug)]
pub enum ShellmError {
    #[error("未配置 API key，请运行: shellm config set-key")]
    MissingApiKey,

    #[error("API 请求失败: {0}")]
    ApiError(String),

    #[error("文件未找到: {0}")]
    FileNotFound(String),

    // TODO: 根据需要添加更多错误变体
}
