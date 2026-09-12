use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct AppError {
    pub code: &'static str,
    pub message: String,
}

impl AppError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new("VALIDATION", message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("NOT_FOUND", message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new("CONFLICT", message)
    }

    pub fn duplicate(message: impl Into<String>) -> Self {
        Self::new("DUPLICATE", message)
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::new("NETWORK", message)
    }

    pub fn startup(message: impl Into<String>) -> Self {
        Self::new("STARTUP", message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("INTERNAL", message)
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", self.code)?;
        state.serialize_field("message", &self.message)?;
        state.end()
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(value: rusqlite::Error) -> Self {
        match &value {
            rusqlite::Error::SqliteFailure(info, _)
                if info.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Self::duplicate("这部作品已经在你的番剧库中")
            }
            _ => Self::internal(format!("数据库错误：{value}")),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::validation(format!("数据格式无效：{value}"))
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::internal(format!("文件错误：{value}"))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        if value.is_timeout() {
            Self::network("请求 Bangumi 超时")
        } else if value.is_connect() {
            Self::network("无法连接 Bangumi")
        } else {
            Self::network(format!("网络错误：{value}"))
        }
    }
}
