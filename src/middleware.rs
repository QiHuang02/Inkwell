use crate::{handlers::auth_middleware, models::AppState};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

/// 应用级认证中间件过滤器，只对需要认证的路径应用认证检查
pub async fn auth_middleware_filter(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path();

    // 定义需要认证的路径模式
    let protected_paths = [
        "/posts", // POST 请求创建文章需要认证
    ];

    let requires_auth = match req.method().as_str() {
        "POST" | "PUT" | "DELETE" => {
            // 对于修改操作，检查是否是受保护的路径
            protected_paths.iter().any(|&p| path.starts_with(p)) || path.contains("/comments") // 所有评论操作需要认证
        }
        _ => false, // GET 请求通常不需要认证
    };

    if requires_auth {
        auth_middleware(State(state), req, next).await
    } else {
        next.run(req).await
    }
}
