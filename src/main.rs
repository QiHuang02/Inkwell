mod config;
mod docs;
mod errors;
mod handlers;
mod middleware;
mod models;
mod routes;
mod utils;
mod validation;

use crate::{
    config::Config, docs::ApiDoc, middleware::auth_middleware_filter, models::AppState,
    routes::create_router,
};
use axum::Router;
use dotenvy::dotenv;
use sqlx::sqlite::SqlitePoolOptions;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // 加载配置
    let config = Config::from_env().expect("Failed to load configuration");

    // 创建数据库连接池
    let pool = SqlitePoolOptions::new()
        .max_connections(config.db_max_connections)
        .connect(&config.database_url)
        .await
        .expect("Can't connect to database");

    let app_state = AppState {
        pool,
        config: config.clone(),
    };

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(create_router())
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware_filter,
        ))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&config.server_address())
        .await
        .unwrap();
    println!("服务已启动，监听地址: http://{}", config.server_address());
    println!(
        "API 文档地址: http://{}/swagger-ui",
        config.server_address()
    );
    axum::serve(listener, app).await.unwrap();
}
