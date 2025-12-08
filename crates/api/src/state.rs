use std::sync::RwLock;
use crate::models::{Quiz, Category};

pub struct AppState {
    pub quizzes: RwLock<Vec<Quiz>>,
    pub categories: RwLock<Vec<Category>>,
}
