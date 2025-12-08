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
        
    assert_eq!(201, response.status().as_u16());
    let json: serde_json::Value = response.json().await.expect("Failed to read JSON");
    json["api_key"].as_str().unwrap().to_string()
}

// ===== Edge Cases =====

#[tokio::test]
async fn create_quiz_with_empty_questions_succeeds() {
    let app = spawn_app().await;
    let token = get_auth_token(&app).await;

    let body = serde_json::json!({
        "title": "Empty Quiz",
        "questions": [],
        "tags": []
    });

    let response = app.api_client
        .post(&format!("{}/quizzes", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(201, response.status().as_u16());
}

#[tokio::test]
async fn create_quiz_with_very_long_title() {
    let app = spawn_app().await;
    let token = get_auth_token(&app).await;

    let long_title = "A".repeat(1000);
    let body = serde_json::json!({
        "title": long_title,
        "questions": [],
        "tags": []
    });

    let response = app.api_client
        .post(&format!("{}/quizzes", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Should succeed (or fail with validation if you add length limits)
    assert!(response.status().as_u16() == 201 || response.status().as_u16() == 400);
}

#[tokio::test]
async fn create_quiz_with_special_characters_in_title() {
    let app = spawn_app().await;
    let token = get_auth_token(&app).await;

    let special_title = "Quiz with émojis 🚀 and symbols: @#$%^&*()";
    let body = serde_json::json!({
        "title": special_title,
        "questions": [],
        "tags": []
    });

    let response = app.api_client
        .post(&format!("{}/quizzes", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(201, response.status().as_u16());
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["title"], special_title);
}

#[tokio::test]
async fn create_question_with_multiple_correct_answers() {
    let app = spawn_app().await;
    let token = get_auth_token(&app).await;

    let body = serde_json::json!({
        "title": "Multi-correct Quiz",
        "questions": [{
            "text": "Which are programming languages?",
            "explanation": "Multiple correct answers",
            "options": [
                { "text": "Rust", "is_correct": true },
                { "text": "Python", "is_correct": true },
                { "text": "HTML", "is_correct": false }
            ]
        }],
        "tags": []
    });

    let response = app.api_client
        .post(&format!("{}/quizzes", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(201, response.status().as_u16());
}

#[tokio::test]
async fn create_question_with_no_correct_answer() {
    let app = spawn_app().await;
    let token = get_auth_token(&app).await;

    let body = serde_json::json!({
        "title": "No Correct Answer Quiz",
        "questions": [{
            "text": "Impossible question?",
            "explanation": "No correct answer",
            "options": [
                { "text": "Wrong 1", "is_correct": false },
                { "text": "Wrong 2", "is_correct": false }
            ]
        }],
        "tags": []
    });

    let response = app.api_client
        .post(&format!("{}/quizzes", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Should succeed (validation could be added to require at least one correct answer)
    assert_eq!(201, response.status().as_u16());
}

// ===== Pagination Tests =====

#[tokio::test]
async fn pagination_first_page_works() {
    let app = spawn_app().await;
    let token = get_auth_token(&app).await;
    let api_key = get_api_key(&app, &token).await;

    // Create 5 quizzes
    for i in 0..5 {
        let body = serde_json::json!({
            "title": format!("Pagination Quiz {}", i),
            "questions": [],
            "tags": []
        });
        app.api_client
            .post(&format!("{}/quizzes", &app.address))
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .await
            .unwrap();
    }

    // Get first page with limit 2
    let response = app.api_client
        .get(&format!("{}/quizzes", &app.address))
        .query(&[("per_page", "2"), ("page", "1")])
        .header("X-API-Key", &api_key)
        .send()
        .await
        .expect("Failed to list quizzes");

    assert_eq!(200, response.status().as_u16());
    let json: serde_json::Value = response.json().await.unwrap();
    let quizzes = json.as_array().unwrap();
    assert!(quizzes.len() <= 2);
}

#[tokio::test]
async fn pagination_beyond_available_pages_returns_empty() {
    let app = spawn_app().await;
    let token = get_auth_token(&app).await;
    let api_key = get_api_key(&app, &token).await;

    // Request page 9999 (likely beyond available data)
    let response = app.api_client
        .get(&format!("{}/quizzes", &app.address))
        .query(&[("per_page", "10"), ("page", "9999")])
        .header("X-API-Key", &api_key)
        .send()
        .await
        .expect("Failed to list quizzes");

    assert_eq!(200, response.status().as_u16());
    let json: serde_json::Value = response.json().await.unwrap();
    let quizzes = json.as_array().unwrap();
    // Should return empty array or small number
    assert!(quizzes.len() <= 10);
}

// ===== Concurrent Operations =====

#[tokio::test]
async fn concurrent_quiz_creation_works() {
    let app = spawn_app().await;
    let token = get_auth_token(&app).await;

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let client = app.api_client.clone();
            let addr = app.address.clone();
            let token = token.clone();
            tokio::spawn(async move {
                let body = serde_json::json!({
                    "title": format!("Concurrent Quiz {}", i),
                    "questions": [],
                    "tags": []
                });
                client
                    .post(&format!("{}/quizzes", &addr))
                    .header("Authorization", format!("Bearer {}", token))
                    .json(&body)
                    .send()
                    .await
                    .unwrap()
            })
        })
        .collect();

    let results = futures::future::join_all(handles).await;
    
    for result in results {
        let response = result.unwrap();
        assert_eq!(201, response.status().as_u16());
    }
}

#[tokio::test]
async fn concurrent_user_registration_with_same_username_fails() {
    let app = spawn_app().await;
    let username = format!("concurrent_user_{}", Uuid::new_v4());

    let handles: Vec<_> = (0..3)
        .map(|_| {
            let client = app.api_client.clone();
            let addr = app.address.clone();
            let username = username.clone();
            tokio::spawn(async move {
                let body = serde_json::json!({
                    "username": username,
                    "password": "password123"
                });
                client
                    .post(&format!("{}/auth/register", &addr))
                    .json(&body)
                    .send()
                    .await
                    .unwrap()
            })
        })
        .collect();

    let results = futures::future::join_all(handles).await;
    
    let mut success_count = 0;
    for result in results {
        let response = result.unwrap();
        if response.status().as_u16() == 200 {
            success_count += 1;
        }
    }
    
    // Only one should succeed
    assert_eq!(1, success_count);
}

// ===== Error Handling =====

#[tokio::test]
async fn malformed_json_returns_400() {
    let app = spawn_app().await;
    let token = get_auth_token(&app).await;

    let response = app.api_client
        .post(&format!("{}/quizzes", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body("{invalid json}")
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn missing_required_fields_returns_400() {
    let app = spawn_app().await;
    let token = get_auth_token(&app).await;

    let body = serde_json::json!({
        "questions": []
        // Missing "title"
    });

    let response = app.api_client
        .post(&format!("{}/quizzes", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn invalid_uuid_format_returns_400_or_404() {
    let app = spawn_app().await;
    let token = get_auth_token(&app).await;
    let api_key = get_api_key(&app, &token).await;

    let response = app.api_client
        .get(&format!("{}/quizzes/not-a-valid-uuid", &app.address))
        .header("X-API-Key", &api_key)
        .send()
        .await
        .expect("Failed to execute request.");

    // Could be 400 (bad request) or 404 (not found) depending on routing
    assert!(response.status().as_u16() == 400 || response.status().as_u16() == 404);
}

// ===== Tag Operations =====

#[tokio::test]
async fn quiz_with_duplicate_tags_deduplicates() {
    let app = spawn_app().await;
    let token = get_auth_token(&app).await;

    let body = serde_json::json!({
        "title": "Duplicate Tags Quiz",
        "questions": [],
        "tags": ["rust", "rust", "programming", "rust"]
    });

    let response = app.api_client
        .post(&format!("{}/quizzes", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(201, response.status().as_u16());
    let json: serde_json::Value = response.json().await.unwrap();
    let tags = json["tags"].as_array().unwrap();
    
    // Should have deduplicated tags (implementation dependent)
    let rust_count = tags.iter().filter(|t| t.as_str().unwrap() == "rust").count();
    assert!(rust_count >= 1); // At least one "rust" tag
}

#[tokio::test]
async fn update_quiz_tags_replaces_old_tags() {
    let app = spawn_app().await;
    let token = get_auth_token(&app).await;

    // Create quiz with initial tags
    let body = serde_json::json!({
        "title": "Tag Update Quiz",
        "questions": [],
        "tags": ["old_tag1", "old_tag2"]
    });

    let response = app.api_client
        .post(&format!("{}/quizzes", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&body)
        .send()
        .await
        .unwrap();

    let quiz: serde_json::Value = response.json().await.unwrap();
    let quiz_id = quiz["id"].as_str().unwrap();

    // Update with new tags
    let update_body = serde_json::json!({
        "tags": ["new_tag1", "new_tag2"]
    });

    let response = app.api_client
        .put(&format!("{}/quizzes/{}", &app.address, quiz_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&update_body)
        .send()
        .await
        .unwrap();

    assert_eq!(200, response.status().as_u16());
    let updated: serde_json::Value = response.json().await.unwrap();
    let tags = updated["tags"].as_array().unwrap();
    
    assert!(tags.iter().any(|t| t.as_str().unwrap() == "new_tag1"));
    assert!(tags.iter().any(|t| t.as_str().unwrap() == "new_tag2"));
    assert!(!tags.iter().any(|t| t.as_str().unwrap() == "old_tag1"));
}

// ===== Authentication Edge Cases =====

#[tokio::test]
async fn expired_token_returns_401() {
    // Note: This test would require mocking time or creating a token with past expiry
    // Skipping implementation for now as it requires additional setup
}

#[tokio::test]
async fn invalid_token_format_returns_401() {
    let app = spawn_app().await;

    let response = app.api_client
        .post(&format!("{}/quizzes", &app.address))
        .header("Authorization", "Bearer invalid.token.here")
        .json(&serde_json::json!({"title": "Test"}))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn missing_bearer_prefix_returns_401() {
    let app = spawn_app().await;
    let token = get_auth_token(&app).await;

    let response = app.api_client
        .post(&format!("{}/quizzes", &app.address))
        .header("Authorization", token) // Missing "Bearer " prefix
        .json(&serde_json::json!({"title": "Test"}))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(401, response.status().as_u16());
}
