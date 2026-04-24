use thiserror::Error;

/// 应用级错误类型
#[derive(Error, Debug)]
pub enum AiError {
    #[error("未配置 API key，请运行: shellm config set-key")]
    MissingApiKey,

    #[error("API 请求失败: {0}")]
    ApiError(String),

    #[error("文件未找到: {0}")]
    FileNotFound(String),
    
    #[error("无效的响应: {reason}, 原始数据: {raw}")]

    InvalidResponse{reason: String, raw: String},

    #[error("HTTP 请求失败: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON 解析失败: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("配置目录未找到")]
    NotFound,

    #[error("配置文件 I/O 失败: {0}")]
    Io(#[from] std::io::Error),         // 读、写、创建目录都用这个

    #[error("配置文件解析失败: {0}")]
    Parse(#[from] serde_json::Error),
}
