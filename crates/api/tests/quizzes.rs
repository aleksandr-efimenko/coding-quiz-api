use crate::common::spawn_app;
use uuid::Uuid;
use coding_quiz_api::id::Id;

mod common;

#[tokio::test]
async fn quiz_crud_works() {
    let app = spawn_app().await;
    
    // 1. Create Quiz
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
        .json(&create_body)
        .send()
        .await
        .expect("Failed to create quiz");

    assert_eq!(201, response.status().as_u16());
    let created_quiz: serde_json::Value = response.json().await.expect("Failed to read JSON");
    let quiz_id = created_quiz["id"].as_str().unwrap();
    assert_eq!(created_quiz["title"], quiz_title);
    
    // 2. Get Quiz
    let response = app.api_client
        .get(&format!("{}/quizzes/{}", &app.address, quiz_id))
        .send()
        .await
        .expect("Failed to get quiz");
    
    assert_eq!(200, response.status().as_u16());
    let fetched_quiz: serde_json::Value = response.json().await.expect("Failed to read JSON");
    assert_eq!(fetched_quiz["id"], quiz_id);
    let tags = fetched_quiz["tags"].as_array().unwrap();
    assert!(tags.iter().any(|t| t.as_str().unwrap() == "test_tag"));

    // 3. Update Quiz
    let update_body = serde_json::json!({
        "title": "Updated Quiz Title",
        "tags": ["updated_tag"]
    });
    
    let response = app.api_client
        .put(&format!("{}/quizzes/{}", &app.address, quiz_id))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to update quiz");
        
    assert_eq!(200, response.status().as_u16());
    let updated_quiz: serde_json::Value = response.json().await.expect("Failed to read JSON");
    assert_eq!(updated_quiz["title"], "Updated Quiz Title");
    
    // 4. Delete Quiz
    let response = app.api_client
        .delete(&format!("{}/quizzes/{}", &app.address, quiz_id))
        .send()
        .await
        .expect("Failed to delete quiz");

    assert_eq!(204, response.status().as_u16());

    // 5. Verify Deletion
    let response = app.api_client
        .get(&format!("{}/quizzes/{}", &app.address, quiz_id))
        .send()
        .await
        .expect("Failed to get quiz");
        
    assert_eq!(404, response.status().as_u16());
}

#[tokio::test]
async fn list_quizzes_filtering_works() {
    let app = spawn_app().await;

    // Create 3 quizzes
    let ids: Vec<String> = futures::future::join_all((0..3).map(|i| {
        let client = &app.api_client;
        let addr = &app.address;
        async move {
            let body = serde_json::json!({
                "title": format!("Quiz {}", i),
                "questions": [],
                "tags": []
            });
            let res = client.post(&format!("{}/quizzes", addr))
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
        .send()
        .await
        .expect("Failed to list quizzes");
    let json: serde_json::Value = response.json().await.unwrap();
    let all_quizzes = json.as_array().unwrap();
    
    for id in &ids {
        assert!(all_quizzes.iter().any(|q| q["id"].as_str().unwrap() == id));
    }

    // List excluding the first one
    let excluded_id = &ids[0];
    let response = app.api_client
        .get(&format!("{}/quizzes", &app.address))
        .query(&[("exclude_ids", excluded_id.as_str()), ("per_page", "100")])
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
    let non_existent_id = Id::new();

    let response = app.api_client
        .get(&format!("{}/quizzes/{}", &app.address, non_existent_id))
        .send()
        .await
        .expect("Failed to execute request.");
    
    assert_eq!(404, response.status().as_u16());
}

#[tokio::test]
async fn delete_non_existent_quiz_fails() {
    let app = spawn_app().await;
    let non_existent_id = Id::new();

    let response = app.api_client
        .delete(&format!("{}/quizzes/{}", &app.address, non_existent_id))
        .send()
        .await
        .expect("Failed to execute request.");
    
    assert_eq!(404, response.status().as_u16());
}

#[tokio::test]
async fn update_non_existent_quiz_fails() {
    let app = spawn_app().await;
    let non_existent_id = Id::new();
    let update_body = serde_json::json!({ "title": "New Title" });

    let response = app.api_client
        .put(&format!("{}/quizzes/{}", &app.address, non_existent_id))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to execute request.");
    
    assert_eq!(404, response.status().as_u16());
}

#[tokio::test]
async fn create_quiz_invalid_data_fails() {
    let app = spawn_app().await;
    // Missing title and questions
    let invalid_body = serde_json::json!({
        "tags": ["test"]
    });

    let response = app.api_client
        .post(&format!("{}/quizzes", &app.address))
        .json(&invalid_body)
        .send()
        .await
        .expect("Failed to execute request.");
    
    // Should be 400 Bad Request due to Json deserialization error
    assert_eq!(400, response.status().as_u16());
}

