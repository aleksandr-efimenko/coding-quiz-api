use crate::id::Id;
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, IntoParams};

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Quiz {
    pub id: Id,
    pub title: String,
    pub category_id: Option<Id>,
    pub questions: Vec<Question>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Question {
    pub id: Id,
    pub text: String,
    pub options: Vec<QuestionOption>,
    pub explanation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
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

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct Category {
    pub id: Id,
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCategoryRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct Tag {
    pub id: Id,
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

