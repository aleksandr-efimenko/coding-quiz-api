use crate::common::spawn_app;
use uuid::Uuid;

mod common;

async fn get_auth_token(app: &common::TestApp) -> String {
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
        
    let json: serde_json::Value = response.json().await.expect("Failed to read JSON");
    json["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn create_and_list_categories_works() {
    let app = spawn_app().await;
    let token = get_auth_token(&app).await;

    // 1. Create Category
    let category_name = format!("Integration Test Category {}", Uuid::new_v4());
    let create_body = serde_json::json!({
        "name": category_name
    });

    let response = app.api_client
        .post(&format!("{}/categories", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&create_body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(201, response.status().as_u16());
    let created: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(created["name"], category_name);

    // 2. List Categories
    let response = app.api_client
        .get(&format!("{}/categories", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
    let list: Vec<serde_json::Value> = response.json().await.expect("Failed to parse JSON list");
    assert!(list.iter().any(|c| c["name"] == category_name));
}

#[tokio::test]
async fn create_duplicate_category_fails() {
    let app = spawn_app().await;
    let token = get_auth_token(&app).await;

    let category_name = format!("Duplicate Test {}", Uuid::new_v4());
    let create_body = serde_json::json!({
        "name": category_name
    });

    // First creation
    let response = app.api_client
        .post(&format!("{}/categories", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&create_body)
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(201, response.status().as_u16());

    // Duplicate creation
    let response = app.api_client
        .post(&format!("{}/categories", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&create_body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().as_u16() != 201);
}
