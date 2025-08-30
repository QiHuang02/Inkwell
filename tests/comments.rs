mod common;
use common::{create_authenticated_user, spawn_app};

async fn create_post(client: &reqwest::Client, app_address: &str, token: &str) -> i64 {
    let post_body =
        serde_json::json!({ "title": "Test Post", "content": "c", "tags": "t", "copyright": "c" });
    let response = client
        .post(&format!("{}/posts", app_address))
        .bearer_auth(token)
        .json(&post_body)
        .send()
        .await
        .unwrap();
    response.json::<serde_json::Value>().await.unwrap()["id"]
        .as_i64()
        .unwrap()
}

async fn create_comment(
    client: &reqwest::Client,
    app_address: &str,
    token: &str,
    post_id: i64,
) -> i64 {
    let comment_body = serde_json::json!({ "content": "Test comment" });
    let response = client
        .post(&format!("{}/posts/{}/comments", app_address, post_id))
        .bearer_auth(token)
        .json(&comment_body)
        .send()
        .await
        .unwrap();
    response.json::<serde_json::Value>().await.unwrap()["id"]
        .as_i64()
        .unwrap()
}

#[tokio::test]
async fn create_comment_returns_a_201_for_valid_authenticated_request() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let user = create_authenticated_user(&client, &app_address).await;
    let post_id = create_post(&client, &app_address, &user.token).await;

    let comment_body = serde_json::json!({ "content": "A great comment" });
    let response = client
        .post(&format!("{}/posts/{}/comments", &app_address, post_id))
        .bearer_auth(&user.token)
        .json(&comment_body)
        .send()
        .await
        .unwrap();

    assert_eq!(201, response.status().as_u16());
    let created_comment: serde_json::Value = response.json().await.unwrap();
    assert_eq!(created_comment["author"], user.username);
}

#[tokio::test]
async fn create_comment_returns_a_401_for_unauthenticated_request() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let user = create_authenticated_user(&client, &app_address).await;
    let post_id = create_post(&client, &app_address, &user.token).await;

    let comment_body = serde_json::json!({ "content": "This should fail" });
    let response = client
        .post(&format!("{}/posts/{}/comments", &app_address, post_id))
        .json(&comment_body)
        .send()
        .await
        .unwrap();

    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn create_comment_returns_a_400_for_invalid_data() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let user = create_authenticated_user(&client, &app_address).await;
    let post_id = create_post(&client, &app_address, &user.token).await;

    let test_cases = vec![
        (serde_json::json!({}), "缺少字段"),
        (serde_json::json!({"content": ""}), "内容为空"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/posts/{}/comments", &app_address, post_id))
            .bearer_auth(&user.token)
            .json(&invalid_body)
            .send()
            .await
            .unwrap();
        let status = response.status().as_u16();
        assert!(
            status == 400 || status == 422,
            "API 在 payload 为 '{}' 时没有返回 400 或 422",
            error_message
        );
    }
}

#[tokio::test]
async fn get_comments_returns_a_list_of_comments_for_a_post() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let user = create_authenticated_user(&client, &app_address).await;
    let post_id = create_post(&client, &app_address, &user.token).await;

    create_comment(&client, &app_address, &user.token, post_id).await;
    create_comment(&client, &app_address, &user.token, post_id).await;

    let response = client
        .get(&format!("{}/posts/{}/comments", &app_address, post_id))
        .send()
        .await
        .unwrap();
    assert_eq!(200, response.status().as_u16());
    let comments: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(2, comments.len());
}

#[tokio::test]
async fn update_comment_returns_a_200_for_valid_authenticated_request() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let user = create_authenticated_user(&client, &app_address).await;
    let post_id = create_post(&client, &app_address, &user.token).await;
    let comment_id = create_comment(&client, &app_address, &user.token, post_id).await;

    let updated_body = serde_json::json!({ "content": "Updated comment" });
    let response = client
        .put(&format!(
            "{}/posts/{}/comments/{}",
            &app_address, post_id, comment_id
        ))
        .bearer_auth(&user.token)
        .json(&updated_body)
        .send()
        .await
        .unwrap();

    assert_eq!(200, response.status().as_u16());
    let updated_comment: serde_json::Value = response.json().await.unwrap();
    assert_eq!("Updated comment", updated_comment["content"]);
}

#[tokio::test]
async fn update_comment_returns_a_403_for_forbidden_request() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let commenter = create_authenticated_user(&client, &app_address).await;
    let post_id = create_post(&client, &app_address, &commenter.token).await;
    let comment_id = create_comment(&client, &app_address, &commenter.token, post_id).await;
    let attacker = create_authenticated_user(&client, &app_address).await;

    let updated_body = serde_json::json!({ "content": "Hacked comment" });
    let response = client
        .put(&format!(
            "{}/posts/{}/comments/{}",
            &app_address, post_id, comment_id
        ))
        .bearer_auth(&attacker.token)
        .json(&updated_body)
        .send()
        .await
        .unwrap();

    assert_eq!(403, response.status().as_u16());
}

#[tokio::test]
async fn delete_comment_returns_a_204_for_valid_authenticated_request() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let user = create_authenticated_user(&client, &app_address).await;
    let post_id = create_post(&client, &app_address, &user.token).await;
    let comment_id = create_comment(&client, &app_address, &user.token, post_id).await;
    let response = client
        .delete(&format!(
            "{}/posts/{}/comments/{}",
            &app_address, post_id, comment_id
        ))
        .bearer_auth(&user.token)
        .send()
        .await
        .unwrap();
    assert_eq!(204, response.status().as_u16());
    let get_response = client
        .get(&format!("{}/posts/{}/comments", &app_address, post_id))
        .send()
        .await
        .unwrap();
    let comments: Vec<serde_json::Value> = get_response.json().await.unwrap();
    assert!(comments.is_empty());
}

#[tokio::test]
async fn update_comment_returns_a_404_if_comment_is_soft_deleted() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let user = create_authenticated_user(&client, &app_address).await;
    let post_id = create_post(&client, &app_address, &user.token).await;
    let comment_id = create_comment(&client, &app_address, &user.token, post_id).await;
    client
        .delete(&format!(
            "{}/posts/{}/comments/{}",
            &app_address, post_id, comment_id
        ))
        .bearer_auth(&user.token)
        .send()
        .await
        .unwrap();
    let updated_body = serde_json::json!({ "content": "Should not update" });
    let update_response = client
        .put(&format!(
            "{}/posts/{}/comments/{}",
            &app_address, post_id, comment_id
        ))
        .bearer_auth(&user.token)
        .json(&updated_body)
        .send()
        .await
        .unwrap();
    assert_eq!(404, update_response.status().as_u16());
}

#[tokio::test]
async fn delete_comment_returns_a_403_for_forbidden_request() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let commenter = create_authenticated_user(&client, &app_address).await;
    let post_id = create_post(&client, &app_address, &commenter.token).await;
    let comment_id = create_comment(&client, &app_address, &commenter.token, post_id).await;
    let attacker = create_authenticated_user(&client, &app_address).await;

    let response = client
        .delete(&format!(
            "{}/posts/{}/comments/{}",
            &app_address, post_id, comment_id
        ))
        .bearer_auth(&attacker.token)
        .send()
        .await
        .unwrap();
    assert_eq!(403, response.status().as_u16());
}
