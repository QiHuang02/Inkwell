use inkwell::{
    config::Config, models::AppState, routes::create_router,
};
use sqlx::SqlitePool;
use tokio::net::TcpListener;

pub async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let config = Config {
        database_url: "sqlite::memory:".to_string(),
        jwt_secret: "test_secret".to_string(),
        server_host: "127.0.0.1".to_string(),
        server_port: port,
        db_max_connections: 1,
        jwt_expiration_days: 1,
    };

    let pool = SqlitePool::connect(&config.database_url)
        .await
        .expect("Failed to connect to SQLite in-memory database.");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate the database.");

    let app_state = AppState {
        pool,
        config: config.clone(),
    };

    let app = create_router(app_state.clone())
        .with_state(app_state);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    address
}

pub struct TestUser {
    pub token: String,
    pub username: String,
}

pub async fn create_authenticated_user(client: &reqwest::Client, app_address: &str) -> TestUser {
    let username = format!(
        "user_{}",
        &uuid::Uuid::new_v4().to_string()[..20].replace('-', "_")
    );
    let password = "password123";

    let register_body = serde_json::json!({ "username": &username, "password": &password });
    let register_response = client
        .post(&format!("{}/register", app_address))
        .json(&register_body)
        .send()
        .await
        .expect("Failed to register user during test setup.");

    if register_response.status().as_u16() != 201 {
        let status = register_response.status();
        let body = register_response.text().await.unwrap();
        panic!(
            "User registration failed in helper. Status: {}. Body: {}",
            status, body
        );
    }

    let login_body = serde_json::json!({ "username": &username, "password": &password });
    let login_response = client
        .post(&format!("{}/login", app_address))
        .json(&login_body)
        .send()
        .await
        .expect("Failed to login user during test setup.");
    assert_eq!(
        200,
        login_response.status().as_u16(),
        "User login failed in helper"
    );

    let login_json: serde_json::Value = login_response.json().await.unwrap();
    let token = login_json["token"].as_str().unwrap().to_string();

    TestUser { token, username }
}
