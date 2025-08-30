use crate::config::Config;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use utoipa::ToSchema;
use validator::Validate;

lazy_static::lazy_static! {
    static ref USERNAME_REGEX: regex::Regex = regex::Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
}

/// 应用的共享状态，包含数据库连接池和配置
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: Config,
}

/// 分页查询参数
#[derive(Deserialize, ToSchema, Validate)]
pub struct Pagination {
    #[serde(default = "default_page")]
    #[validate(range(min = 1, max = 1000, message = "页码必须在 1-1000 之间"))]
    pub page: u64,
    #[serde(default = "default_page_size")]
    #[validate(range(min = 1, max = 100, message = "每页数量必须在 1-100 之间"))]
    pub page_size: u64,
}

/// `page` 的默认值函数
pub fn default_page() -> u64 {
    1
}

/// `page_size` 的默认值函数
pub fn default_page_size() -> u64 {
    10
}

/// 文章的数据模型
#[derive(Serialize, Deserialize, Clone, sqlx::FromRow, ToSchema)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub author_id: i64,
    pub content: String,
    pub tags: String,
    pub copyright: String,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// 用于API响应的文章结构，包含作者用户名
#[derive(Serialize, ToSchema, sqlx::FromRow)]
pub struct PostResponse {
    pub id: i64,
    pub title: String,
    pub author: String,
    pub content: String,
    pub tags: String,
    pub copyright: String,
    pub created_at: DateTime<Utc>,
}

/// 评论的数据模型
#[derive(Serialize, Deserialize, Clone, sqlx::FromRow, ToSchema)]
pub struct Comment {
    #[sqlx(rename = "comment_id")]
    pub id: i64,
    pub post_id: i64,
    pub author_id: i64,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// 用于API响应的评论结构，包含作者用户名
#[derive(Serialize, ToSchema, sqlx::FromRow)]
pub struct CommentResponse {
    pub id: i64,
    pub post_id: i64,
    pub author: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

/// 创建新文章时接收的数据
#[derive(Deserialize, Clone, ToSchema, Validate)]
pub struct CreatePost {
    #[validate(length(min = 1, max = 200, message = "标题长度必须在 1-200 字符之间"))]
    pub title: String,
    #[validate(length(min = 1, max = 10000, message = "内容长度必须在 1-10000 字符之间"))]
    pub content: String,
    #[validate(length(max = 200, message = "标签长度不能超过 200 字符"))]
    pub tags: String,
    #[validate(length(max = 200, message = "版权信息长度不能超过 200 字符"))]
    pub copyright: String,
}

/// 创建新评论时接收的数据
#[derive(Deserialize, Clone, ToSchema, Validate)]
pub struct CreateComment {
    #[validate(length(min = 1, max = 1000, message = "评论内容长度必须在 1-1000 字符之间"))]
    pub content: String,
}

// --- 新增的用户认证相关模型 ---

/// 用户的数据模型，直接映射数据库的 `users` 表
/// 注意：这个结构体不应该被序列化返回给客户端，因为它包含密码哈希
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    #[allow(dead_code)]
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub role: String,
}

/// 用户注册时接收的数据
#[derive(Deserialize, ToSchema, Validate)]
pub struct RegisterUser {
    #[schema(example = "new_user")]
    #[validate(length(min = 3, max = 30, message = "用户名长度必须在 3-30 字符之间"))]
    #[validate(regex(path = "*USERNAME_REGEX", message = "用户名只能包含字母、数字和下划线"))]
    pub username: String,
    #[schema(example = "password123")]
    #[validate(length(min = 6, max = 100, message = "密码长度必须在 6-100 字符之间"))]
    pub password: String,
}

/// 用户登录时接收的数据
#[derive(Deserialize, ToSchema, Validate)]
pub struct LoginUser {
    #[schema(example = "new_user")]
    #[validate(length(min = 3, max = 30, message = "用户名长度必须在 3-30 字符之间"))]
    pub username: String,
    #[schema(example = "password123")]
    #[validate(length(min = 1, max = 100, message = "密码不能为空且不能超过 100 字符"))]
    pub password: String,
}

/// 登录成功后返回的 JWT 令牌
#[derive(Serialize, ToSchema)]
pub struct TokenResponse {
    #[schema(example = "a.very.long.jwt.token.string")]
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

/// 分页响应结构
#[derive(Serialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub page: u64,
    pub page_size: u64,
    pub total: u64,
    pub total_pages: u64,
}
