mod common;
use common::{create_authenticated_user, spawn_app, TestUser};

// 辅助函数：创建一个帖子并返回其 ID
async fn create_post(client: &reqwest::Client, app_address: &str, token: &str) -> i64 {
    let post_body = serde_json::json!({
        "title": "Test Post", "content": "Test content", "tags": "test", "copyright": "test"
    });
    let response = client
        .post(&format!("{}/posts", app_address))
        .bearer_auth(token)
        .json(&post_body)
        .send()
        .await
        .unwrap();
    let post_json: serde_json::Value = response.json().await.unwrap();
    post_json["id"].as_i64().unwrap()
}

// 辅助函数：为授权测试创建作者和攻击者
async fn create_author_and_attacker(
    client: &reqwest::Client,
    app_address: &str,
) -> (TestUser, i64, TestUser) {
    let author = create_authenticated_user(client, app_address).await;
    let post_id = create_post(client, app_address, &author.token).await;
    let attacker = create_authenticated_user(client, app_address).await;
    (author, post_id, attacker)
}

#[tokio::test]
async fn create_post_returns_a_201_for_valid_authenticated_request() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let user = create_authenticated_user(&client, &app_address).await;

    let post_body = serde_json::json!({
        "title": "My First Post", "content": "Content", "tags": "tags", "copyright": "copyright"
    });

    let response = client
        .post(&format!("{}/posts", &app_address))
        .bearer_auth(&user.token)
        .json(&post_body)
        .send()
        .await
        .expect("Failed to create post.");

    assert_eq!(201, response.status().as_u16());
    let created_post: serde_json::Value = response.json().await.unwrap();
    assert_eq!(created_post["author"], user.username);
}

#[tokio::test]
async fn create_post_returns_a_401_for_unauthenticated_request() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let post_body = serde_json::json!({ "title": "Unauthorized", "content": "c", "tags": "t", "copyright": "c" });

    let response = client
        .post(&format!("{}/posts", &app_address))
        .json(&post_body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn create_post_returns_a_400_for_invalid_data() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let user = create_authenticated_user(&client, &app_address).await;

    let test_cases = vec![
        (serde_json::json!({"content": "c"}), "缺少标题"),
        (serde_json::json!({"title": "t"}), "缺少内容"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/posts", &app_address))
            .bearer_auth(&user.token)
            .json(&invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        let status = response.status().as_u16();
        assert!(
            status == 400 || status == 422,
            "API 在 payload 为 '{}' 时没有返回 400 或 422",
            error_message
        );
    }
}

#[tokio::test]
async fn get_posts_returns_a_list_of_posts() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let user = create_authenticated_user(&client, &app_address).await;

    create_post(&client, &app_address, &user.token).await;
    create_post(&client, &app_address, &user.token).await;

    let response = client
        .get(&format!("{}/posts", &app_address))
        .send()
        .await
        .unwrap();
    assert_eq!(200, response.status().as_u16());
    let json_body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(2, json_body["data"].as_array().unwrap().len());
}

#[tokio::test]
async fn update_post_returns_a_200_for_valid_authenticated_request() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let user = create_authenticated_user(&client, &app_address).await;
    let post_id = create_post(&client, &app_address, &user.token).await;

    let updated_body = serde_json::json!({ "title": "Updated", "content": "Updated", "tags": "u", "copyright": "u" });

    let response = client
        .put(&format!("{}/posts/{}", &app_address, post_id))
        .bearer_auth(&user.token)
        .json(&updated_body)
        .send()
        .await
        .unwrap();

    assert_eq!(200, response.status().as_u16());
    let updated_post: serde_json::Value = response.json().await.unwrap();
    assert_eq!("Updated", updated_post["title"]);
}

#[tokio::test]
async fn update_post_returns_a_403_for_forbidden_request() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let (_author, post_id, attacker) = create_author_and_attacker(&client, &app_address).await;

    let updated_body =
        serde_json::json!({ "title": "Hacked", "content": "h", "tags": "h", "copyright": "h" });
    let response = client
        .put(&format!("{}/posts/{}", &app_address, post_id))
        .bearer_auth(&attacker.token)
        .json(&updated_body)
        .send()
        .await
        .unwrap();

    assert_eq!(403, response.status().as_u16());
}

#[tokio::test]
async fn delete_post_returns_a_204_for_valid_authenticated_request() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let user = create_authenticated_user(&client, &app_address).await;
    let post_id = create_post(&client, &app_address, &user.token).await;

    let response = client
        .delete(&format!("{}/posts/{}", &app_address, post_id))
        .bearer_auth(&user.token)
        .send()
        .await
        .unwrap();

    assert_eq!(204, response.status().as_u16());

    let get_response = client
        .get(&format!("{}/posts/{}", &app_address, post_id))
        .send()
        .await
        .unwrap();
    assert_eq!(404, get_response.status().as_u16());
}

#[tokio::test]
async fn delete_post_returns_a_403_for_forbidden_request() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let (_author, post_id, attacker) = create_author_and_attacker(&client, &app_address).await;

    let response = client
        .delete(&format!("{}/posts/{}", &app_address, post_id))
        .bearer_auth(&attacker.token)
        .send()
        .await
        .unwrap();

    assert_eq!(403, response.status().as_u16());
}

#[tokio::test]
async fn update_post_returns_a_404_if_post_is_soft_deleted() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let user = create_authenticated_user(&client, &app_address).await;
    let post_id = create_post(&client, &app_address, &user.token).await;

    client
        .delete(&format!("{}/posts/{}", &app_address, post_id))
        .bearer_auth(&user.token)
        .send()
        .await
        .unwrap();

    let updated_body = serde_json::json!({ "title": "Should not update", "content": "s", "tags": "s", "copyright": "s" });

    let update_response = client
        .put(&format!("{}/posts/{}", &app_address, post_id))
        .bearer_auth(&user.token)
        .json(&updated_body)
        .send()
        .await
        .unwrap();

    assert_eq!(404, update_response.status().as_u16());
}
