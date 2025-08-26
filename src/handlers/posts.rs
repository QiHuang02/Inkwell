use crate::{
    errors::{AppError, ErrorResponse},
    models::{AppState, CreatePost, PaginatedResponse, Pagination, Post},
    utils::{check_delete_result, created_response},
    validation::{format_validation_errors, ValidatedJson},
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use validator::Validate;

#[utoipa::path(
    get,
    path = "/posts",
    params(
        ("page" = Option<u64>, Query, description = "页码"),
        ("page_size" = Option<u64>, Query, description = "每页数量")
    ),
    responses(
        (status = 200, description = "成功列出所有文章", body = PaginatedResponse<Post>),
        (status = 500, description = "内部服务器错误", body = ErrorResponse)
    ),
    tag = "Posts"
)]
pub async fn get_posts(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<PaginatedResponse<Post>>, AppError> {
    // 验证分页参数
    pagination.validate().map_err(|validation_errors| {
        AppError::validation(format!(
            "分页参数{}",
            format_validation_errors(&validation_errors)
        ))
    })?;

    // 获取总数
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts")
        .fetch_one(&state.pool)
        .await?;
    let total = total.0 as u64;

    // 计算分页信息
    let offset = (pagination.page - 1) * pagination.page_size;
    let total_pages = total.div_ceil(pagination.page_size);

    // 获取分页数据
    let posts = sqlx::query_as::<_, Post>("SELECT * FROM posts ORDER BY id LIMIT ? OFFSET ?")
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
        (status = 201, description = "成功创建文章", body = Post),
        (status = 500, description = "内部服务器错误", body = ErrorResponse)
    ),
    tag = "Posts"
)]
pub async fn create_post(
    State(state): State<AppState>,
    json_payload: Json<CreatePost>,
) -> Result<impl IntoResponse, AppError> {
    let payload = json_payload.validate_json()?;
    let post = sqlx::query_as::<_, Post>(
        "INSERT INTO posts (title, author, content, tags, copyright) VALUES (?, ?, ?, ?, ?) RETURNING *",
    )
        .bind(&payload.title)
        .bind(&payload.author)
        .bind(&payload.content)
        .bind(&payload.tags)
        .bind(&payload.copyright)
        .fetch_one(&state.pool)
        .await?;
    Ok(created_response(post))
}

#[utoipa::path(
    get,
    path = "/posts/{id}",
    params(("id" = u64, Path, description = "文章 ID")),
    responses(
        (status = 200, description = "根据 ID 获取文章", body = Post),
        (status = 404, description = "未找到文章", body = ErrorResponse)
    ),
    tag = "Posts"
)]
pub async fn get_post_by_id(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Post>, AppError> {
    let post = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = ?")
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
        (status = 200, description = "成功更新文章", body = Post),
        (status = 404, description = "未找到文章", body = ErrorResponse)
    ),
    tag = "Posts"
)]
pub async fn update_post(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    json_payload: Json<CreatePost>,
) -> Result<Json<Post>, AppError> {
    let payload = json_payload.validate_json()?;
    let post = sqlx::query_as::<_, Post>(
        "UPDATE posts SET title = ?, author = ?, content = ?, tags = ?, copyright = ? WHERE id = ? RETURNING *",
    )
        .bind(&payload.title)
        .bind(&payload.author)
        .bind(&payload.content)
        .bind(&payload.tags)
        .bind(&payload.copyright)
        .bind(id as i64)
        .fetch_one(&state.pool)
        .await?;
    Ok(Json(post))
}

#[utoipa::path(
    delete,
    path = "/posts/{id}",
    params(("id" = u64, Path, description = "文章 ID")),
    responses(
        (status = 204, description = "成功删除文章"),
        (status = 404, description = "未找到文章", body = ErrorResponse)
    ),
    tag = "Posts"
)]
pub async fn delete_post(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM posts WHERE id = ?")
        .bind(id as i64)
        .execute(&state.pool)
        .await?;
    check_delete_result(result, "Post")
}
