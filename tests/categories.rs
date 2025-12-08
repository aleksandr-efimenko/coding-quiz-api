use crate::common::spawn_app;
use uuid::Uuid;
use crate::common::{get_auth_token, get_api_key};

mod common;

#[tokio::test]
async fn create_and_list_categories_works() {
    let app = spawn_app().await;
    let token = common::get_auth_token(&app).await;

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
    let api_key = common::get_api_key(&app, &token).await;
    let response = app.api_client
        .get(&format!("{}/categories", &app.address))
        .header("X-API-Key", api_key)
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
