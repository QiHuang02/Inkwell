use crate::errors::AppError;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

/// 检查删除操作的结果，如果没有行被影响则返回 NotFound 错误
pub fn check_delete_result(
    result: sqlx::sqlite::SqliteQueryResult,
    resource_name: &str,
) -> Result<StatusCode, AppError> {
    if result.rows_affected() > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("{} not found", resource_name)))
    }
}

/// 创建标准的创建响应 (201 Created + JSON)
pub fn created_response<T>(data: T) -> impl IntoResponse
where
    T: Serialize,
{
    (StatusCode::CREATED, Json(data))
}

/// 执行密码哈希操作
pub async fn hash_password(password: &str) -> Result<String, AppError> {
    let password_clone = password.to_string();
    tokio::task::spawn_blocking(move || bcrypt::hash(&password_clone, bcrypt::DEFAULT_COST))
        .await?
        .map_err(|_| AppError::PasswordHash)
}

/// 验证密码
pub async fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let password_clone = password.to_string();
    let hash_clone = hash.to_string();
    tokio::task::spawn_blocking(move || bcrypt::verify(&password_clone, &hash_clone))
        .await?
        .map_err(|_| AppError::PasswordHash)
}
