use actix_web::{web, HttpResponse, Responder};
use crate::models::{
    CreateQuizRequest, Quiz, Question, QuestionOption, 
    SubmitAnswerRequest, AnswerResponse,
    Category, CreateCategoryRequest, UpdateQuizRequest, Tag,
    PaginationParams, ErrorResponse,
};
use crate::state::AppState;
use crate::id::Id;
use rand::Rng;

#[utoipa::path(
    get,
    path = "/health",
    tag = "System",
    responses(
        (status = 200, description = "Health Check", body = String)
    )
)]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[utoipa::path(
    post,
    path = "/quizzes",
    request_body = CreateQuizRequest,
    tag = "Management",
    responses(
        (status = 201, description = "Quiz created", body = Quiz),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn create_quiz(
    data: web::Data<AppState>,
    req: web::Json<CreateQuizRequest>,
) -> impl Responder {
    let mut quizzes = match data.quizzes.write() {
        Ok(q) => q,
        Err(_) => return HttpResponse::InternalServerError().body("Lock poisoned"),
    };

    let quiz_id = Id::new();
    let questions = req.questions.iter().map(|q| {
        let q_id = Id::new();
        let options = q.options.iter().map(|o| {
            QuestionOption {
                id: Id::new(),
                text: o.text.clone(),
                is_correct: o.is_correct,
                description: o.description.clone(),
            }
        }).collect();
        Question {
            id: q_id,
            text: q.text.clone(),
            options,
            explanation: q.explanation.clone(),
        }
    }).collect();

    let new_quiz = Quiz {
        id: quiz_id,
        title: req.title.clone(),
        category_id: req.category_id,
        questions,
        tags: req.tags.clone().unwrap_or_default(),
    };

    quizzes.push(new_quiz.clone());

    HttpResponse::Created().json(new_quiz)
}

#[utoipa::path(
    get,
    path = "/quizzes/{id}",
    tag = "Consumption",
    params(
        ("id" = Id, Path, description = "Quiz ID")
    ),
    responses(
        (status = 200, description = "Get Quiz by ID", body = Quiz),
        (status = 404, description = "Quiz not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn get_quiz(
    data: web::Data<AppState>,
    path: web::Path<Id>,
) -> impl Responder {
    let quizzes = match data.quizzes.read() {
        Ok(q) => q,
        Err(_) => return HttpResponse::InternalServerError().body("Lock poisoned"),
    };

    let quiz_id = path.into_inner();
    if let Some(quiz) = quizzes.iter().find(|q| q.id == quiz_id) {
        HttpResponse::Ok().json(quiz)
    } else {
        HttpResponse::NotFound().body("Quiz not found")
    }
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct ListQuizzesFilter {
    category_id: Option<Id>,
    pub exclude_ids: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[utoipa::path(
    get,
    path = "/quizzes",
    tag = "Consumption",
    params(
        ListQuizzesFilter
    ),
    responses(
        (status = 200, description = "List Quizzes", body = Vec<Quiz>),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn list_quizzes(
    data: web::Data<AppState>, 
    filter: web::Query<ListQuizzesFilter>,
) -> impl Responder {
    let quizzes = match data.quizzes.read() {
        Ok(q) => q,
        Err(_) => return HttpResponse::InternalServerError().json(ErrorResponse { error: "Lock poisoned".to_string() }),
    };

    let page = filter.page.unwrap_or(1);
    let per_page = filter.per_page.unwrap_or(10);
    
    // Convert exclude_ids string to Vec<Id>
    let exclude_ids: Vec<Id> = filter.exclude_ids.as_deref().unwrap_or("")
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    let filtered: Vec<Quiz> = quizzes.iter()
        .filter(|q| {
            if let Some(cat_id) = filter.category_id {
                if q.category_id != Some(cat_id) { return false; }
            }
            if !exclude_ids.is_empty() {
                if exclude_ids.contains(&q.id) { return false; }
            }
            true
        })
        .cloned()
        .collect();

    // Pagination
    let total = filtered.len();
    let start = ((page - 1) * per_page) as usize;
    if start >= total {
         return HttpResponse::Ok().json(Vec::<Quiz>::new());
    }
    let end = std::cmp::min(start + per_page as usize, total);
    
    let page_items = &filtered[start..end];
    HttpResponse::Ok().json(page_items)
}

#[utoipa::path(
    post,
    path = "/quizzes/{id}/solve",
    request_body = SubmitAnswerRequest,
    tag = "Consumption",
    params(
        ("id" = Id, Path, description = "Quiz ID")
    ),
    responses(
        (status = 200, description = "Answer result", body = AnswerResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn submit_answer(
    data: web::Data<AppState>,
    _path: web::Path<Id>,
    req: web::Json<SubmitAnswerRequest>,
) -> impl Responder {
    let quizzes = match data.quizzes.read() {
        Ok(q) => q,
        Err(_) => return HttpResponse::InternalServerError().json(ErrorResponse{ error: "Lock poisoned".to_string() }),
    };

    // Find question
    for quiz in quizzes.iter() {
        if let Some(question) = quiz.questions.iter().find(|q| q.id == req.question_id) {
            if let Some(option) = question.options.iter().find(|o| o.id == req.option_id) {
                return HttpResponse::Ok().json(AnswerResponse {
                    correct: option.is_correct,
                    message: if option.is_correct { "Correct!".to_string() } else { "Incorrect.".to_string() },
                    explanation: question.explanation.clone(),
                });
            }
        }
    }

    HttpResponse::BadRequest().json(ErrorResponse{ error: "Invalid question or option".to_string() })
}


#[utoipa::path(
    post,
    path = "/categories",
    request_body = CreateCategoryRequest,
    tag = "Management",
    responses(
        (status = 201, description = "Category created", body = Category),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn create_category(
    data: web::Data<AppState>,
    req: web::Json<CreateCategoryRequest>,
) -> impl Responder {
    let mut categories = match data.categories.write() {
        Ok(c) => c,
        Err(_) => return HttpResponse::InternalServerError().body("Lock poisoned"),
    };

    let id = Id::new();
    let new_category = Category { id, name: req.name.clone() };
    categories.push(new_category.clone());
    
    HttpResponse::Created().json(new_category)
}

#[utoipa::path(
    get,
    path = "/categories",
    tag = "Consumption",
    params(
        PaginationParams
    ),
    responses(
        (status = 200, description = "List Categories", body = Vec<Category>),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn list_categories(
    data: web::Data<AppState>, 
    filter: web::Query<PaginationParams>,
) -> impl Responder {
    let categories = match data.categories.read() {
        Ok(c) => c,
        Err(_) => return HttpResponse::InternalServerError().json(ErrorResponse { error: "Lock poisoned".to_string() }),
    };

    let page = filter.page.unwrap_or(1);
    let per_page = filter.per_page.unwrap_or(10);

    let total = categories.len();
    let start = ((page - 1) * per_page) as usize;
    if start >= total {
        return HttpResponse::Ok().json(Vec::<Category>::new());
    }
    let end = std::cmp::min(start + per_page as usize, total);

    let page_items: Vec<Category> = categories.iter().skip(start).take(per_page as usize).cloned().collect();
    HttpResponse::Ok().json(page_items)
}

#[utoipa::path(
    delete,
    path = "/quizzes/{id}",
    tag = "Management",
    params(
        ("id" = Id, Path, description = "Quiz ID")
    ),
    responses(
        (status = 204, description = "Quiz deleted"),
        (status = 404, description = "Quiz not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn delete_quiz(
    data: web::Data<AppState>,
    path: web::Path<Id>,
) -> impl Responder {
    let mut quizzes = match data.quizzes.write() {
        Ok(q) => q,
        Err(_) => return HttpResponse::InternalServerError().body("Lock poisoned"),
    };
    
    let id = path.into_inner();
    let initial_len = quizzes.len();
    quizzes.retain(|q| q.id != id);
    
    if quizzes.len() < initial_len {
        HttpResponse::NoContent().finish()
    } else {
        HttpResponse::NotFound().body("Quiz not found")
    }
}

#[utoipa::path(
    put,
    path = "/quizzes/{id}",
    request_body = UpdateQuizRequest,
    tag = "Management",
    params(
        ("id" = Id, Path, description = "Quiz ID")
    ),
    responses(
        (status = 200, description = "Quiz updated", body = Quiz),
        (status = 404, description = "Quiz not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn update_quiz(
    data: web::Data<AppState>,
    path: web::Path<Id>,
    req: web::Json<UpdateQuizRequest>,
) -> impl Responder {
    let mut quizzes = match data.quizzes.write() {
        Ok(q) => q,
        Err(_) => return HttpResponse::InternalServerError().body("Database error"),
    };
    
    let id = path.into_inner();
    if let Some(quiz) = quizzes.iter_mut().find(|q| q.id == id) {
        if let Some(title) = &req.title {
            quiz.title = title.clone();
        }
        if let Some(cat_id) = req.category_id {
            quiz.category_id = Some(cat_id);
        }
        if let Some(tags) = &req.tags {
            quiz.tags = tags.clone();
        }
         HttpResponse::Ok().json(quiz.clone())
    } else {
         HttpResponse::NotFound().body("Quiz not found")
    }
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct RandomQuizParams {
    pub tag: Option<String>,
    pub user_email: Option<String>, // Ignored in memory for history tracking
}

#[utoipa::path(
    get,
    path = "/quizzes/random",
    tag = "Consumption",
    params(
        RandomQuizParams
    ),
    responses(
        (status = 200, description = "Random Quiz", body = Quiz),
        (status = 404, description = "No quizzes found"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn get_random_quiz(
    data: web::Data<AppState>,
    params: web::Query<RandomQuizParams>,
) -> impl Responder {
    let quizzes = match data.quizzes.read() {
        Ok(q) => q,
        Err(_) => return HttpResponse::InternalServerError().body("Lock poisoned"),
    };

    let filtered: Vec<&Quiz> = quizzes.iter().filter(|q| {
        if let Some(tag) = &params.tag {
            return q.tags.contains(tag);
        }
        true
    }).collect();

    if filtered.is_empty() {
        return HttpResponse::NotFound().body("No quizzes found");
    }

    let mut rng = rand::thread_rng();
    let random_index = rng.gen_range(0..filtered.len());
    let quiz = filtered[random_index];

    HttpResponse::Ok().json(quiz)
}
