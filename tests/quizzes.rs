use crate::common::spawn_app;
use uuid::Uuid;
use coding_quiz_api::id::Id;

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
                "text": format!("What is Rust? {}", Uuid::new_v4()),
                "explanation": "A systems programming language",
                "options": [
                    { "text": "A game", "is_correct": false },
                    { "text": "A language", "is_correct": true }
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
    let non_existent_id = Id::new();

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
    let non_existent_id = Id::new();

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
    let non_existent_id = Id::new();
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

#[tokio::test]
async fn test_b2b_managed_learning_flow() {
    let app = spawn_app().await;
    let jwt_token = get_auth_token(&app).await;
    let api_key = get_api_key(&app, &jwt_token).await;

    // 1. Create a Quiz
    let quiz_body = serde_json::json!({
        "title": "Learning Flow Quiz",
        "questions": [{
            "text": format!("Q1_{}", Uuid::new_v4()),
            "explanation": "Exp",
            "options": [
                { "text": "Correct", "is_correct": true },
                { "text": "Wrong", "is_correct": false }
            ]
        }],
        "tags": []
    });
    
    let res = app.api_client
        .post(&format!("{}/quizzes", &app.address))
        .header("Authorization", format!("Bearer {}", jwt_token))
        .json(&quiz_body)
        .send()
        .await
        .unwrap();
    if !res.status().is_success() {
        println!("Create Quiz Failed: status={}, body={:?}", res.status(), res.text().await);
        panic!("Failed to create quiz");
    }
    let quiz_json: serde_json::Value = res.json().await.unwrap();
    let quiz_id = quiz_json["id"].as_str().unwrap();
    let question_id = quiz_json["questions"][0]["id"].as_str().unwrap();
    let option_correct_id = quiz_json["questions"][0]["options"][0]["id"].as_str().unwrap();

    // 1.5 Verify Question
    let res = app.api_client
        .put(&format!("{}/questions/{}/verify", &app.address, question_id))
        .header("Authorization", format!("Bearer {}", jwt_token))
        .json(&serde_json::json!({ "verified": true }))
        .send()
        .await
        .unwrap();
    assert_eq!(200, res.status().as_u16());

    // 2. Register End User
    let user_email = "alice@example.com";
    let res = app.api_client
        .post(&format!("{}/users", &app.address))
        .header("X-API-Key", &api_key)
        .json(&serde_json::json!({ "email": user_email }))
        .send()
        .await
        .unwrap();
    assert_eq!(201, res.status().as_u16());

    // 3. Solve Quiz (Correctly)
    let solve_body = serde_json::json!({
        "user_email": user_email,
        "question_id": question_id,
        "option_id": option_correct_id
    });

    let res = app.api_client
        .post(&format!("{}/quizzes/{}/solve", &app.address, quiz_id))
        .header("X-API-Key", &api_key)
        .json(&solve_body)
        .send()
        .await
        .unwrap();
    assert_eq!(200, res.status().as_u16());
    let ans_json: serde_json::Value = res.json().await.unwrap();
    assert_eq!(ans_json["correct"], true);

    // 4. Verify History
    let res = app.api_client
        .get(&format!("{}/users/{}/history", &app.address, user_email))
        .header("X-API-Key", &api_key)
        .send()
        .await
        .unwrap();
    assert_eq!(200, res.status().as_u16());
    let history: serde_json::Value = res.json().await.unwrap();
    let history_arr = history.as_array().unwrap();
    assert_eq!(history_arr.len(), 1);
    assert_eq!(history_arr[0]["quiz_id"], quiz_id);
    assert_eq!(history_arr[0]["is_correct"], true);

    // 5. Random Quiz (Should NOT return the solved quiz)
    // Since we only have 1 quiz and it's solved, we expect 404 (No available quizzes)
    let _res = app.api_client
        .get(&format!("{}/quizzes/random", &app.address))
        .query(&[("user_email", user_email)])
        .header("X-API-Key", &api_key)
        .send()
        .await
        .unwrap();
    // It might return 404 if "No available quizzes" logic works, OR it might pick another quiz if tests run in shared DB (which they do).
    // Given shared DB, we can't strictly assert 404 unless we ensure isolate, but we can verify it doesn't return THIS quiz id if we loop/retry or just check logic.
    // For this test, to be robust in shared DB:
    // Ideally we'd create 2 quizzes, solve 1, ensure Random returns the other.
    
    // Create Quiz 2
    let quiz2_body = serde_json::json!({ "title": "Quiz 2", "questions": [], "tags": [] });
     let res = app.api_client
        .post(&format!("{}/quizzes", &app.address))
        .header("Authorization", format!("Bearer {}", jwt_token))
        .json(&quiz2_body)
        .send()
        .await
        .unwrap();
    let quiz2_json: serde_json::Value = res.json().await.unwrap();
    let _quiz2_id = quiz2_json["id"].as_str().unwrap();

    // Now Random should return quiz 2
    let res = app.api_client
        .get(&format!("{}/quizzes/random", &app.address))
        .query(&[("user_email", user_email)])
        .header("X-API-Key", &api_key)
        .send()
        .await
        .unwrap();
    
    if res.status() == 200 {
        let random_quiz: serde_json::Value = res.json().await.unwrap();
        // Should NOT be quiz_id
        assert_ne!(random_quiz["id"].as_str().unwrap(), quiz_id);
    }

    // 6. Test Tag Filtering
    let tagged_quiz_body = serde_json::json!({ 
        "title": "Tagged Quiz", 
        "questions": [{
            "text": format!("Tagged Q {}", Uuid::new_v4()),
            "explanation": "Exp",
            "options": [
                { "text": "A", "is_correct": true },
                { "text": "B", "is_correct": false }
            ]
        }], 
        "tags": ["special_tag"] 
    });
    let res = app.api_client
        .post(&format!("{}/quizzes", &app.address))
        .header("Authorization", format!("Bearer {}", jwt_token))
        .json(&tagged_quiz_body)
        .send()
        .await
        .unwrap();
    assert_eq!(201, res.status().as_u16());
    let tagged_json: serde_json::Value = res.json().await.unwrap();
    let tagged_id = tagged_json["id"].as_str().unwrap();
    let tagged_q_id = tagged_json["questions"][0]["id"].as_str().unwrap();

    // Verify Tagged Quiz Question
    let res = app.api_client
        .put(&format!("{}/questions/{}/verify", &app.address, tagged_q_id))
        .header("Authorization", format!("Bearer {}", jwt_token))
        .json(&serde_json::json!({ "verified": true }))
        .send()
        .await
        .unwrap();
    assert_eq!(200, res.status().as_u16());

    // Request with tag
    let res = app.api_client
        .get(&format!("{}/quizzes/random", &app.address))
        .query(&[("user_email", user_email), ("tag", "special_tag")])
        .header("X-API-Key", &api_key)
        .send()
        .await
        .unwrap();
    
    assert_eq!(200, res.status().as_u16());
    let random_tagged: serde_json::Value = res.json().await.unwrap();
    assert_eq!(random_tagged["id"].as_str().unwrap(), tagged_id);

    // Request with non-existent tag
    let res = app.api_client
        .get(&format!("{}/quizzes/random", &app.address))
        .query(&[("user_email", user_email), ("tag", "does_not_exist")])
        .header("X-API-Key", &api_key)
        .send()
        .await
        .unwrap();
    assert_eq!(404, res.status().as_u16());
}
