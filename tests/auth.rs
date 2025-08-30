mod common;
use common::spawn_app;

#[tokio::test]
async fn register_returns_a_201_for_valid_form_data() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "username": "test_user",
        "password": "password123"
    });

    let response = client
        .post(&format!("{}/register", app_address))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 201);
}

#[tokio::test]
async fn register_returns_a_422_when_fields_are_missing() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        (serde_json::json!({"username": "test_user"}), "缺少密码"),
        (serde_json::json!({"password": "password123"}), "缺少用户名"),
        (serde_json::json!({}), "缺少所有字段"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/register", app_address))
            .header("Content-Type", "application/json")
            .json(&invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            422,
            response.status().as_u16(),
            "API 在 payload 为 '{}' 时没有返回 422",
            error_message
        );
    }
}

#[tokio::test]
async fn register_returns_a_400_when_fields_are_invalid() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        (
            serde_json::json!({"username": "u", "password": "password123"}),
            "用户名太短",
        ),
        (
            serde_json::json!({"username": "test_user", "password": "123"}),
            "密码太短",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/register", app_address))
            .header("Content-Type", "application/json")
            .json(&invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "API 在 payload 为 '{}' 时没有返回 400",
            error_message
        );
    }
}

#[tokio::test]
async fn register_returns_a_409_when_username_is_taken() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "username": "test_user_conflict",
        "password": "password123"
    });

    let response1 = client
        .post(&format!("{}/register", app_address))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response1.status().as_u16(), 201);

    let response2 = client
        .post(&format!("{}/register", app_address))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response2.status().as_u16(), 409);
}

#[tokio::test]
async fn login_returns_a_200_and_token_for_valid_credentials() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();

    let username = "testuser_login_success";
    let password = "password123";

    let register_body = serde_json::json!({
        "username": &username,
        "password": &password
    });

    let response_register = client
        .post(&format!("{}/register", &app_address))
        .header("Content-Type", "application/json")
        .json(&register_body)
        .send()
        .await
        .expect("Failed to execute register request.");
    assert_eq!(response_register.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "username": &username,
        "password": &password
    });

    let response_login = client
        .post(&format!("{}/login", &app_address))
        .header("Content-Type", "application/json")
        .json(&login_body)
        .send()
        .await
        .expect("Failed to execute login request.");

    assert_eq!(response_login.status().as_u16(), 200);

    let json_body: serde_json::Value = response_login
        .json()
        .await
        .expect("Failed to parse login response to JSON.");

    assert!(
        json_body["token"].as_str().is_some(),
        "响应体中没有找到 token 字段"
    );
}

#[tokio::test]
async fn login_returns_a_401_for_invalid_credentials() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let username = "testuser_login_fail";
    let password = "correct_password";

    let register_body = serde_json::json!({
        "username": username,
        "password": password
    });
    let response_register = client
        .post(&format!("{}/register", &app_address))
        .header("Content-Type", "application/json")
        .json(&register_body)
        .send()
        .await
        .expect("Failed to execute register request.");
    assert_eq!(
        response_register.status().as_u16(),
        201,
        "User registration failed"
    );

    let test_cases = vec![
        (
            serde_json::json!({"username": "wrong_user", "password": password}),
            "错误的用户名",
        ),
        (
            serde_json::json!({"username": username, "password": "wrong_password"}),
            "错误的密码",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/login", &app_address))
            .header("Content-Type", "application/json")
            .json(&invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            401,
            response.status().as_u16(),
            "API 在 payload 为 '{}' 时没有返回 401",
            error_message
        );
    }
}
