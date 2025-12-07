use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{ToSchema, IntoParams};

#[derive(Debug, Serialize, Deserialize, Clone, FromRow, ToSchema)]
pub struct Quiz {
    pub id: uuid::Uuid,
    pub title: String,
    pub category_id: Option<uuid::Uuid>,
    #[sqlx(skip)]
    pub questions: Vec<Question>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow, ToSchema)]
pub struct Question {
    pub id: uuid::Uuid,
    pub text: String,
    #[sqlx(skip)]
    pub options: Vec<QuestionOption>,
    pub explanation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow, ToSchema)]
pub struct QuestionOption {
    pub id: uuid::Uuid,
    pub text: String,
    #[serde(skip_serializing)] 
    pub is_correct: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateQuizRequest {
    pub title: String,
    pub category_id: Option<uuid::Uuid>,
    pub questions: Vec<CreateQuestionRequest>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateQuizRequest {
    pub title: Option<String>,
    pub category_id: Option<uuid::Uuid>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateQuestionRequest {
    pub text: String,
    pub options: Vec<CreateOptionRequest>,
    pub explanation: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOptionRequest {
    pub text: String,
    pub is_correct: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SubmitAnswerRequest {
    pub question_id: uuid::Uuid,
    pub option_id: uuid::Uuid,
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
    pub id: uuid::Uuid,
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
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Category {
    pub id: uuid::Uuid,
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCategoryRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Tag {
    pub id: uuid::Uuid,
    pub name: String,
}
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: uuid::Uuid,
    pub developer_id: uuid::Uuid,
    pub key_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct UsageLog {
    pub id: uuid::Uuid,
    pub api_key_id: uuid::Uuid,
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
    pub id: uuid::Uuid,
    pub username: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}
