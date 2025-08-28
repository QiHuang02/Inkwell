use crate::{errors::{AppError, ErrorResponse}, models::{AppState, Claims, Comment, CommentResponse, CreateComment, User}, utils::{check_delete_result, created_response}, validation::ValidatedJson};
use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json, Extension};

#[utoipa::path(
    get,
    path = "/posts/{id}/comments",
    params(("id" = u64, Path, description = "文章 ID")),
    responses(
        (status = 200, description = "列出文章的所有评论", body = [CommentResponse])
    ),
    tag = "Comments"
)]
pub async fn get_comments_for_post(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Vec<CommentResponse>>, AppError> {
    let comments = sqlx::query_as::<_, CommentResponse>(
        "SELECT c.comment_id as id, c.post_id, u.username as author, c.content, c.created_at FROM comments c JOIN users u ON c.author_id = u.id WHERE c.post_id = ?",
    )
    .bind(id as i64)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(comments))
}

#[utoipa::path(
    post,
    path = "/posts/{id}/comments",
    params(("id" = u64, Path, description = "文章 ID")),
    request_body = CreateComment,
    responses(
        (status = 201, description = "成功创建评论", body = CommentResponse)
    ),
    tag = "Comments",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_comment_for_post(
    State(state): State<AppState>,
    Path(post_id): Path<u64>,
    Extension(claims): Extension<Claims>,
    json_payload: Json<CreateComment>,
) -> Result<impl IntoResponse, AppError> {
    let payload = json_payload.validate_json()?;

    let user: User = sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(&claims.sub)
        .fetch_one(&state.pool)
        .await?;

    let comment = sqlx::query_as::<_, Comment>(
        "INSERT INTO comments (post_id, author_id, content) VALUES (?, ?, ?) RETURNING *",
    )
    .bind(post_id as i64)
    .bind(user.id)
    .bind(&payload.content)
    .fetch_one(&state.pool)
    .await?;

    let comment_response = CommentResponse {
        id: comment.id,
        post_id: comment.post_id,
        author: user.username,
        content: comment.content,
        created_at: comment.created_at,
    };

    Ok(created_response(comment_response))
}

#[utoipa::path(
    put,
    path = "/posts/{post_id}/comments/{comment_id}",
    params(
        ("post_id" = u64, Path, description = "文章 ID"),
        ("comment_id" = u64, Path, description = "评论 ID")
    ),
    request_body = CreateComment,
    responses(
        (status = 200, description = "成功更新评论", body = CommentResponse),
        (status = 403, description = "无权限操作", body = ErrorResponse),
        (status = 404, description = "未找到评论", body = ErrorResponse)
    ),
    tag = "Comments",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_comment(
    State(state): State<AppState>,
    Path((post_id, comment_id)): Path<(u64, u64)>,
    Extension(claims): Extension<Claims>,
    json_payload: Json<CreateComment>,
) -> Result<Json<CommentResponse>, AppError> {
    let payload = json_payload.validate_json()?;

    let user: User = sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(&claims.sub)
        .fetch_one(&state.pool)
        .await?;

    let comment: Comment = sqlx::query_as("SELECT * FROM comments WHERE comment_id = ? AND post_id = ?")
        .bind(comment_id as i64)
        .bind(post_id as i64)
        .fetch_one(&state.pool)
        .await?;

    if comment.author_id != user.id {
        return Err(AppError::authorization("无权限修改此评论"));
    }

    let updated_comment = sqlx::query_as::<_, Comment>(
        "UPDATE comments SET content = ? WHERE comment_id = ? AND post_id = ? RETURNING *",
    )
    .bind(&payload.content)
    .bind(comment_id as i64)
    .bind(post_id as i64)
    .fetch_one(&state.pool)
    .await?;

    let comment_response = CommentResponse {
        id: updated_comment.id,
        post_id: updated_comment.post_id,
        author: user.username,
        content: updated_comment.content,
        created_at: updated_comment.created_at,
    };

    Ok(Json(comment_response))
}

#[utoipa::path(
    delete,
    path = "/posts/{post_id}/comments/{comment_id}",
    params(
        ("post_id" = u64, Path, description = "文章 ID"),
        ("comment_id" = u64, Path, description = "评论 ID")
    ),
    responses(
        (status = 204, description = "成功删除评论"),
        (status = 403, description = "无权限操作", body = ErrorResponse),
        (status = 404, description = "未找到评论", body = ErrorResponse)
    ),
    tag = "Comments",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_comment(
    State(state): State<AppState>,
    Path((post_id, comment_id)): Path<(u64, u64)>,
    Extension(claims): Extension<Claims>,
) -> Result<StatusCode, AppError> {
    let user: User = sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(&claims.sub)
        .fetch_one(&state.pool)
        .await?;

    let comment: Comment = sqlx::query_as("SELECT * FROM comments WHERE comment_id = ? AND post_id = ?")
        .bind(comment_id as i64)
        .bind(post_id as i64)
        .fetch_one(&state.pool)
        .await?;

    if comment.author_id != user.id {
        return Err(AppError::authorization("无权限删除此评论"));
    }

    let result = sqlx::query("DELETE FROM comments WHERE comment_id = ?")
        .bind(comment_id as i64)
        .execute(&state.pool)
        .await?;

    check_delete_result(result, "Comment")
}
