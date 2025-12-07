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

async fn get_api_key(app: &common::TestApp, token: &str) -> String {
    let response = app.api_client
        .post(&format!("{}/auth/api-keys", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request.");
        
    // 201 Created
    assert_eq!(201, response.status().as_u16());
    let json: serde_json::Value = response.json().await.expect("Failed to read JSON");
    json["api_key"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn quiz_crud_works() {
    let app = spawn_app().await;
    let jwt_token = get_auth_token(&app).await;
    let api_key = get_api_key(&app, &jwt_token).await;
    
    // 1. Create Quiz (Management - JWT)
    let quiz_title = "Integration Test Quiz";
    let create_body = serde_json::json!({
        "title": quiz_title,
        "category_id": null,
        "questions": [
             {
                "text": "Question 1",
                "explanation": "Explanation 1",
                "options": [
                    { "text": "Option 1", "is_correct": true },
                    { "text": "Option 2", "is_correct": false }
                ]
            }
        ],
        "tags": ["test_tag"]
    });

    let response = app.api_client
        .post(&format!("{}/quizzes", &app.address))
        .header("Authorization", format!("Bearer {}", jwt_token))
        .json(&create_body)
        .send()
        .await
        .expect("Failed to create quiz");

    assert_eq!(201, response.status().as_u16());
    let created_quiz: serde_json::Value = response.json().await.expect("Failed to read JSON");
    let quiz_id = created_quiz["id"].as_str().unwrap();
    assert_eq!(created_quiz["title"], quiz_title);
    
    // 2. Get Quiz (Consumption - API Key)
    // NOTE: Used to be Bearer, now X-API-Key
    let response = app.api_client
        .get(&format!("{}/quizzes/{}", &app.address, quiz_id))
        .header("X-API-Key", &api_key)
        .send()
        .await
        .expect("Failed to get quiz");
    
    assert_eq!(200, response.status().as_u16());
    let fetched_quiz: serde_json::Value = response.json().await.expect("Failed to read JSON");
    assert_eq!(fetched_quiz["id"], quiz_id);
    let tags = fetched_quiz["tags"].as_array().unwrap();
    assert!(tags.iter().any(|t| t.as_str().unwrap() == "test_tag"));

    // 3. Update Quiz (Management - JWT)
    let update_body = serde_json::json!({
        "title": "Updated Quiz Title",
        "tags": ["updated_tag"]
    });
    
    let response = app.api_client
        .put(&format!("{}/quizzes/{}", &app.address, quiz_id))
        .header("Authorization", format!("Bearer {}", jwt_token))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to update quiz");
        
    assert_eq!(200, response.status().as_u16());
    let updated_quiz: serde_json::Value = response.json().await.expect("Failed to read JSON");
    assert_eq!(updated_quiz["title"], "Updated Quiz Title");
    
    // 4. Delete Quiz (Management - JWT)
    let response = app.api_client
        .delete(&format!("{}/quizzes/{}", &app.address, quiz_id))
        .header("Authorization", format!("Bearer {}", jwt_token))
        .send()
        .await
        .expect("Failed to delete quiz");

    assert_eq!(204, response.status().as_u16());

    // 5. Verify Deletion (Consumption - API Key)
    let response = app.api_client
        .get(&format!("{}/quizzes/{}", &app.address, quiz_id))
        .header("X-API-Key", &api_key)
        .send()
        .await
        .expect("Failed to get quiz");
        
    assert_eq!(404, response.status().as_u16());
}

#[tokio::test]
async fn list_quizzes_filtering_works() {
    let app = spawn_app().await;
    let jwt_token = get_auth_token(&app).await;
    let api_key = get_api_key(&app, &jwt_token).await;

    // Create 3 quizzes
    let ids: Vec<String> = futures::future::join_all((0..3).map(|i| {
        let client = &app.api_client;
        let addr = &app.address;
        let token = &jwt_token;
        async move {
            let body = serde_json::json!({
                "title": format!("Quiz {}", i),
                "questions": [],
                "tags": []
            });
            let res = client.post(&format!("{}/quizzes", addr))
                .header("Authorization", format!("Bearer {}", token))
                .json(&body)
                .send()
                .await
                .unwrap();
            let json: serde_json::Value = res.json().await.unwrap();
            json["id"].as_str().unwrap().to_string()
        }
    })).await;

    // List all
    let response = app.api_client
        .get(&format!("{}/quizzes", &app.address))
        .query(&[("per_page", "100")])
        .header("X-API-Key", &api_key)
        .send()
        .await
        .expect("Failed to list quizzes");
    let json: serde_json::Value = response.json().await.unwrap();
    let all_quizzes = json.as_array().unwrap();
    // Ensure we see at least our 3 quizzes (could be more if DB not clean)
    // We can't guarantee count if reusing DB, but we can check existence
    for id in &ids {
        assert!(all_quizzes.iter().any(|q| q["id"].as_str().unwrap() == id));
    }

    // List excluding the first one
    let excluded_id = &ids[0];
    let response = app.api_client
        .get(&format!("{}/quizzes", &app.address))
        .query(&[("exclude_ids", excluded_id.as_str()), ("per_page", "100")])
        .header("X-API-Key", &api_key)
        .send()
        .await
        .expect("Failed to list quizzes");
    
    let json: serde_json::Value = response.json().await.unwrap();
    let filtered_quizzes = json.as_array().unwrap();
    
    assert!(!filtered_quizzes.iter().any(|q| q["id"].as_str().unwrap() == excluded_id));
    assert!(filtered_quizzes.iter().any(|q| q["id"].as_str().unwrap() == &ids[1]));
    assert!(filtered_quizzes.iter().any(|q| q["id"].as_str().unwrap() == &ids[2]));
}

#[tokio::test]
async fn get_non_existent_quiz_fails() {
    let app = spawn_app().await;
    let token = get_auth_token(&app).await;
    let api_key = get_api_key(&app, &token).await;
    let non_existent_id = Uuid::new_v4();

    let response = app.api_client
        .get(&format!("{}/quizzes/{}", &app.address, non_existent_id))
        .header("X-API-Key", api_key)
        .send()
        .await
        .expect("Failed to execute request.");
    
    assert_eq!(404, response.status().as_u16());
}

#[tokio::test]
async fn delete_non_existent_quiz_fails() {
    let app = spawn_app().await;
    let token = get_auth_token(&app).await;
    let non_existent_id = Uuid::new_v4();

    let response = app.api_client
        .delete(&format!("{}/quizzes/{}", &app.address, non_existent_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request.");
    
    assert_eq!(404, response.status().as_u16());
}

#[tokio::test]
async fn update_non_existent_quiz_fails() {
    let app = spawn_app().await;
    let token = get_auth_token(&app).await;
    let non_existent_id = Uuid::new_v4();
    let update_body = serde_json::json!({ "title": "New Title" });

    let response = app.api_client
        .put(&format!("{}/quizzes/{}", &app.address, non_existent_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to execute request.");
    
    assert_eq!(404, response.status().as_u16());
}

#[tokio::test]
async fn create_quiz_invalid_data_fails() {
    let app = spawn_app().await;
    let token = get_auth_token(&app).await;
    // Missing title and questions
    let invalid_body = serde_json::json!({
        "tags": ["test"]
    });

    let response = app.api_client
        .post(&format!("{}/quizzes", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&invalid_body)
        .send()
        .await
        .expect("Failed to execute request.");
    
    // Should be 400 Bad Request due to Json deserialization error
    assert_eq!(400, response.status().as_u16());
}
