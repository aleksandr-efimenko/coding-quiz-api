use crate::common::spawn_app;
use uuid::Uuid;

mod common;

#[tokio::test]
async fn create_and_list_categories_works() {
    let app = spawn_app().await;

    // 1. Create Category
    let category_name = format!("Integration Test Category {}", Uuid::new_v4());
    let create_body = serde_json::json!({
        "name": category_name
    });

    let response = app.api_client
        .post(&format!("{}/categories", &app.address))
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
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
    let _list: Vec<serde_json::Value> = response.json().await.expect("Failed to parse JSON list");
}

