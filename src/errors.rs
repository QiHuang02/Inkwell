use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    #[error("验证错误: {0}")]
    Validation(String),

    #[error("认证错误: {message}")]
    Authentication { message: String },

    #[error("授权错误: {message}")]
    Authorization { message: String },

    #[error("未找到资源: {message}")]
    NotFound { message: String },

    #[error("冲突错误: {message}")]
    Conflict { message: String },

    #[error("内部服务器错误: {message}")]
    Internal { message: String },

    #[error("密码哈希错误")]
    PasswordHash,

    #[error("JWT错误: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("任务执行错误")]
    TaskJoin(#[from] tokio::task::JoinError),
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    #[schema(example = "验证失败")]
    pub error: String,
    #[schema(example = 400)]
    pub code: u16,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Database(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Authentication { .. } => StatusCode::UNAUTHORIZED,
            AppError::Authorization { .. } => StatusCode::FORBIDDEN,
            AppError::NotFound { .. } => StatusCode::NOT_FOUND,
            AppError::Conflict { .. } => StatusCode::CONFLICT,
            AppError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::PasswordHash => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Jwt(_) => StatusCode::UNAUTHORIZED,
            AppError::TaskJoin(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_message(&self) -> String {
        match self {
            AppError::Database(sqlx::Error::RowNotFound) => "请求的资源未找到".to_string(),
            AppError::Database(_) => "数据库操作失败".to_string(),
            _ => self.to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status_code();
        let error_response = ErrorResponse {
            error: self.error_message(),
            code: status.as_u16(),
        };

        (status, Json(error_response)).into_response()
    }
}

// 便捷构造函数
impl AppError {
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    pub fn authentication(msg: impl Into<String>) -> Self {
        Self::Authentication {
            message: msg.into(),
        }
    }

    pub fn authorization(msg: impl Into<String>) -> Self {
        Self::Authorization {
            message: msg.into(),
        }
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound {
            message: msg.into(),
        }
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict {
            message: msg.into(),
        }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal {
            message: msg.into(),
        }
    }
}
