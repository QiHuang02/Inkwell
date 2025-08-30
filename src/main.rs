use axum::Router;
use dotenvy::dotenv;
use inkwell::{create_router, docs::ApiDoc, middleware::auth_middleware_filter, AppState, Config};
use sqlx::sqlite::SqlitePoolOptions;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "inkwell=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载配置
    tracing::info!("Loading config......");
    let config = Config::from_env().expect("Failed to load configuration");

    // 创建数据库连接池
    tracing::info!("Loading database......");
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
    tracing::info!("服务已启动，监听地址: http://{}", config.server_address());
    tracing::info!(
        "API 文档地址: http://{}/swagger-ui",
        config.server_address()
    );
    axum::serve(listener, app).await.unwrap();
}
