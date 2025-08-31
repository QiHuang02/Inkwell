use crate::handlers::*;
use crate::models::AppState;
use axum::{
    routing::{get, post, put},
    Router,
};

#[utoipa::path(
    get,
    path = "/",
    responses((status = 200, description = "根路径")),
    tag = "Rust Blog API"
)]
pub async fn root() -> &'static str {
    "Hello, World!"
}

/// 创建一个总路由函数，供 main.rs 调用
pub fn create_router(app_state: AppState) -> Router<AppState> {
    let protected_routes = Router::new()
        .route(
            "/posts",
            post(create_post),
        )
        .route("/posts/{id}", put(update_post).delete(delete_post))
        .route("/posts/{id}/comments", post(create_comment_for_post))
        .route(
            "/posts/{post_id}/comments/{comment_id}",
            put(update_comment).delete(delete_comment),
        )
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    let public_routes = Router::new()
        .route("/", get(root))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/posts", get(get_posts))
        .route("/posts/{id}", get(get_post_by_id))
        .route("/posts/{id}/comments", get(get_comments_for_post));

    public_routes.merge(protected_routes)
}
