use crate::common::spawn_app;
use uuid::Uuid;

mod common;

#[tokio::test]
async fn auth_register_and_login_works() {
    let app = spawn_app().await;
    
    // 1. Register
    let username = format!("user_{}", Uuid::new_v4());
    let password = "password123";

    let register_body = serde_json::json!({
        "username": username,
        "password": password
    });

    let response = app.api_client
        .post(&format!("{}/auth/register", &app.address))
        .json(&register_body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    // 2. Login
    let login_body = serde_json::json!({
        "username": username,
        "password": password
    });

    let response = app.api_client
        .post(&format!("{}/auth/login", &app.address))
        .json(&login_body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
    
    let json: serde_json::Value = response.json().await.expect("Failed to read JSON");
    assert!(json.get("token").is_some());
}

#[tokio::test]
async fn auth_login_fails_with_wrong_password() {
    let app = spawn_app().await;
    
    // Register
    let username = format!("user_{}", Uuid::new_v4());
    let password = "password123";

    let register_body = serde_json::json!({
        "username": username,
        "password": password
    });
    app.api_client
        .post(&format!("{}/auth/register", &app.address))
        .json(&register_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Login with wrong password
    let login_body = serde_json::json!({
        "username": username,
        "password": "wrongpassword"
    });

    let response = app.api_client
        .post(&format!("{}/auth/login", &app.address))
        .json(&login_body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn auth_register_duplicate_username_fails() {
    let app = spawn_app().await;
    let username = format!("user_{}", Uuid::new_v4());
    let password = "password123";

    let register_body = serde_json::json!({
        "username": username,
        "password": password
    });

    // First registration
    let response = app.api_client
        .post(&format!("{}/auth/register", &app.address))
        .json(&register_body)
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(200, response.status().as_u16());

    // Duplicate registration
    let response = app.api_client
        .post(&format!("{}/auth/register", &app.address))
        .json(&register_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assuming API returns 500 or 409 for duplicate. 
    // SQLx usually throws a violation which results in 500 Internal Server Error with current handler implementation.
    // Ideally it should be 409 Conflict, but checking for failure (not 200) is enough for now.
    assert!(response.status().as_u16() != 200);
}

#[tokio::test]
async fn auth_login_non_existent_user_fails() {
    let app = spawn_app().await;
    let login_body = serde_json::json!({
        "username": "non_existent_user_12345",
        "password": "password123"
    });

    let response = app.api_client
        .post(&format!("{}/auth/login", &app.address))
        .json(&login_body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn access_protected_route_without_token_fails() {
    let app = spawn_app().await;

    let response = app.api_client
        .post(&format!("{}/quizzes", &app.address))
        .json(&serde_json::json!({"title": "Test"}))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(401, response.status().as_u16());
}
