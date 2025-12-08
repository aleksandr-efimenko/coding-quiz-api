use crate::common::spawn_app;

mod common;

async fn get_quiz_data(app: &common::TestApp) -> (String, String, String, String) {
    // 1. Create Quiz
    let create_body = serde_json::json!({
        "title": "Exec Quiz",
        "category_id": null,
        "questions": [
             {
                "text": "Exec Q1",
                "explanation": "Exec Explanation",
                "options": [
                    { "text": "Correct Opt", "is_correct": true },
                    { "text": "Wrong Opt", "is_correct": false }
                ]
            }
        ],
        "tags": []
    });

    let q_res = app.api_client
        .post(&format!("{}/quizzes", &app.address))
        .json(&create_body)
        .send()
        .await
        .expect("Failed to create quiz");
    let q_json: serde_json::Value = q_res.json().await.unwrap();
    let quiz_id = q_json["id"].as_str().unwrap().to_string();
    
    // Get question and option IDs from response
    let questions = q_json["questions"].as_array().unwrap();
    let question_obj = &questions[0];
    let question_id = question_obj["id"].as_str().unwrap().to_string();
    let options = question_obj["options"].as_array().unwrap();
    
    // Find IDs by text
    let correct_id = options.iter().find(|o| o["text"] == "Correct Opt").unwrap()["id"].as_str().unwrap().to_string();
    let wrong_id = options.iter().find(|o| o["text"] == "Wrong Opt").unwrap()["id"].as_str().unwrap().to_string();
    
    (quiz_id, question_id, correct_id, wrong_id)
}

#[tokio::test]
async fn submit_correct_answer_returns_true() {
    let app = spawn_app().await;
    let (quiz_id, question_id, correct_id, _) = get_quiz_data(&app).await;

    let submit_body = serde_json::json!({
        "question_id": question_id,
        "option_id": correct_id
    });

    let response = app.api_client
        .post(&format!("{}/quizzes/{}/solve", &app.address, quiz_id))
        .json(&submit_body)
        .send()
        .await
        .expect("Failed to submit");

    assert_eq!(200, response.status().as_u16());
    let res_json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(res_json["correct"], true);
    assert_eq!(res_json["explanation"], "Exec Explanation");
}

#[tokio::test]
async fn submit_incorrect_answer_returns_false() {
    let app = spawn_app().await;
    let (quiz_id, question_id, _, wrong_id) = get_quiz_data(&app).await;

    let submit_body = serde_json::json!({
        "question_id": question_id,
        "option_id": wrong_id
    });

    let response = app.api_client
        .post(&format!("{}/quizzes/{}/solve", &app.address, quiz_id))
        .json(&submit_body)
        .send()
        .await
        .expect("Failed to submit");

    assert_eq!(200, response.status().as_u16());
    let res_json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(res_json["correct"], false);
    // Explanation should still be returned
    assert_eq!(res_json["explanation"], "Exec Explanation");
}

