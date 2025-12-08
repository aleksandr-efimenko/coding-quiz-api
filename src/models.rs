use crate::id::Id;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{ToSchema, IntoParams};

#[derive(Debug, Serialize, Deserialize, Clone, FromRow, ToSchema)]
pub struct Quiz {
    pub id: Id,
    pub title: String,
    pub category_id: Option<Id>,
    #[sqlx(skip)]
    pub questions: Vec<Question>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow, ToSchema)]
pub struct Question {
    pub id: Id,
    pub text: String,
    #[sqlx(skip)]
    pub options: Vec<QuestionOption>,
    pub explanation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow, ToSchema)]
pub struct QuestionOption {
    pub id: Id,
    pub text: String,
    #[serde(skip_serializing)] 
    pub is_correct: bool,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateQuizRequest {
    pub title: String,
    pub category_id: Option<Id>,
    pub questions: Vec<CreateQuestionRequest>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateQuizRequest {
    pub title: Option<String>,
    pub category_id: Option<Id>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateQuestionRequest {
    pub text: String,
    pub options: Vec<CreateOptionRequest>,
    pub explanation: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct CreateOptionRequest {
    pub text: String,
    pub is_correct: bool,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SubmitAnswerRequest {
    pub question_id: Id,
    pub option_id: Id,
    pub user_email: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AnswerResponse {
    pub correct: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Developer {
    pub id: Id,
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub developer_id: i64, // Needs to be numeric for JWT usually, but let's check auth.rs usage. 
                           // But for now replacing with known structure.
                           // Actually Claims usually doesn't need to match generic Id but let's keep it simple.
                           // Wait, auth.rs defines how it's used. I should check auth.rs before committing to this change in Claims.
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Category {
    pub id: Id,
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCategoryRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Tag {
    pub id: Id,
    pub name: String,
}
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: Id,
    pub developer_id: Id,
    pub key_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct UsageLog {
    pub id: Id,
    pub api_key_id: Id,
    pub endpoint: String,
    pub status_code: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeveloperResponse {
    pub id: Id,
    pub username: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct EndUser {
    pub id: Id,
    pub email: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateEndUserRequest {
    pub email: String,
}

#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct UserAnswerHistory {
    pub quiz_id: Id,
    pub question_id: Id,
    pub option_id: Id,
    pub is_correct: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
