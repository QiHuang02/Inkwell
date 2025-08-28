use crate::{errors::{AppError, ErrorResponse}, models::{AppState, Claims, CreatePost, PaginatedResponse, Pagination, Post, PostResponse, User}, utils::{check_delete_result, created_response}, validation::{format_validation_errors, ValidatedJson}};
use axum::{extract::{Path, Query, State}, http::StatusCode, response::IntoResponse, Json, Extension};
use validator::Validate;

#[utoipa::path(
    get,
    path = "/posts",
    params(
        ("page" = Option<u64>, Query, description = "页码"),
        ("page_size" = Option<u64>, Query, description = "每页数量")
    ),
    responses(
        (status = 200, description = "成功列出所有文章", body = PaginatedResponse<PostResponse>),
        (status = 500, description = "内部服务器错误", body = ErrorResponse)
    ),
    tag = "Posts"
)]
pub async fn get_posts(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<PaginatedResponse<PostResponse>>, AppError> {
    pagination.validate().map_err(|validation_errors| {
        AppError::validation(format!(
            "分页参数{}",
            format_validation_errors(&validation_errors)
        ))
    })?;

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts")
        .fetch_one(&state.pool)
        .await?;
    let total = total.0 as u64;

    let offset = (pagination.page - 1) * pagination.page_size;
    let total_pages = total.div_ceil(pagination.page_size);

    let posts = sqlx::query_as::<_, PostResponse>(
        "SELECT p.*, u.username as author FROM posts p JOIN users u ON p.author_id = u.id ORDER BY p.id LIMIT ? OFFSET ?",
    )
    .bind(pagination.page_size as i64)
    .bind(offset as i64)
    .fetch_all(&state.pool)
    .await?;

    let response = PaginatedResponse {
        data: posts,
        page: pagination.page,
        page_size: pagination.page_size,
        total,
        total_pages,
    };

    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/posts",
    request_body = CreatePost,
    responses(
        (status = 201, description = "成功创建文章", body = PostResponse),
        (status = 500, description = "内部服务器错误", body = ErrorResponse)
    ),
    tag = "Posts",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_post(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    json_payload: Json<CreatePost>,
) -> Result<impl IntoResponse, AppError> {
    let payload = json_payload.validate_json()?;

    let user: User = sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(&claims.sub)
        .fetch_one(&state.pool)
        .await?;

    let post = sqlx::query_as::<_, Post>(
        "INSERT INTO posts (title, author_id, content, tags, copyright) VALUES (?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(&payload.title)
    .bind(user.id)
    .bind(&payload.content)
    .bind(&payload.tags)
    .bind(&payload.copyright)
    .fetch_one(&state.pool)
    .await?;

    let post_response = PostResponse {
        id: post.id,
        title: post.title,
        author: user.username,
        content: post.content,
        tags: post.tags,
        copyright: post.copyright,
        created_at: post.created_at,
    };

    Ok(created_response(post_response))
}

#[utoipa::path(
    get,
    path = "/posts/{id}",
    params(("id" = u64, Path, description = "文章 ID")),
    responses(
        (status = 200, description = "根据 ID 获取文章", body = PostResponse),
        (status = 404, description = "未找到文章", body = ErrorResponse)
    ),
    tag = "Posts"
)]
pub async fn get_post_by_id(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<PostResponse>, AppError> {
    let post = sqlx::query_as::<_, PostResponse>(
        "SELECT p.*, u.username as author FROM posts p JOIN users u ON p.author_id = u.id WHERE p.id = ?",
    )
    .bind(id as i64)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(post))
}

#[utoipa::path(
    put,
    path = "/posts/{id}",
    params(("id" = u64, Path, description = "文章 ID")),
    request_body = CreatePost,
    responses(
        (status = 200, description = "成功更新文章", body = PostResponse),
        (status = 403, description = "无权限操作", body = ErrorResponse),
        (status = 404, description = "未找到文章", body = ErrorResponse)
    ),
    tag = "Posts",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_post(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Extension(claims): Extension<Claims>,
    json_payload: Json<CreatePost>,
) -> Result<Json<PostResponse>, AppError> {
    let payload = json_payload.validate_json()?;

    let user: User = sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(&claims.sub)
        .fetch_one(&state.pool)
        .await?;

    let post: Post = sqlx::query_as("SELECT * FROM posts WHERE id = ?")
        .bind(id as i64)
        .fetch_one(&state.pool)
        .await?;

    if post.author_id != user.id {
        return Err(AppError::authorization("无权限修改此文章"));
    }

    let updated_post = sqlx::query_as::<_, Post>(
        "UPDATE posts SET title = ?, content = ?, tags = ?, copyright = ? WHERE id = ? RETURNING *",
    )
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(&payload.tags)
    .bind(&payload.copyright)
    .bind(id as i64)
    .fetch_one(&state.pool)
    .await?;

    let post_response = PostResponse {
        id: updated_post.id,
        title: updated_post.title,
        author: user.username,
        content: updated_post.content,
        tags: updated_post.tags,
        copyright: updated_post.copyright,
        created_at: updated_post.created_at,
    };

    Ok(Json(post_response))
}

#[utoipa::path(
    delete,
    path = "/posts/{id}",
    params(("id" = u64, Path, description = "文章 ID")),
    responses(
        (status = 204, description = "成功删除文章"),
        (status = 403, description = "无权限操作", body = ErrorResponse),
        (status = 404, description = "未找到文章", body = ErrorResponse)
    ),
    tag = "Posts",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_post(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Extension(claims): Extension<Claims>,
) -> Result<StatusCode, AppError> {
    let user: User = sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(&claims.sub)
        .fetch_one(&state.pool)
        .await?;

    let post: Post = sqlx::query_as("SELECT * FROM posts WHERE id = ?")
        .bind(id as i64)
        .fetch_one(&state.pool)
        .await?;

    if post.author_id != user.id {
        return Err(AppError::authorization("无权限删除此文章"));
    }

    let result = sqlx::query("DELETE FROM posts WHERE id = ?")
        .bind(id as i64)
        .execute(&state.pool)
        .await?;

    check_delete_result(result, "Post")
}
