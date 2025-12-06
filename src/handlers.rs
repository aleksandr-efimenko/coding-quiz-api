use actix_web::{web, HttpResponse, Responder};
use crate::models::{
    CreateQuizRequest, Quiz, Question, QuestionOption, 
    SubmitAnswerRequest, AnswerResponse,
    RegisterRequest, LoginRequest, TokenResponse, User,
    Category, CreateCategoryRequest
};
use crate::state::AppState;
use crate::auth;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/health",
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
    responses(
        (status = 201, description = "Quiz created", body = Quiz),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn create_quiz(
    data: web::Data<AppState>,
    req: web::Json<CreateQuizRequest>,
    _: auth::JwtMiddleware, // Require auth
) -> impl Responder {
    let quiz_id = Uuid::new_v4();
    
    // Start transaction
    let mut tx = match data.db.begin().await {
        Ok(tx) => tx,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to start transaction"),
    };

    if let Err(_) = sqlx::query!(
        "INSERT INTO quizzes (id, title, category_id) VALUES ($1, $2, $3)",
        quiz_id,
        req.title,
        req.category_id
    )
    .execute(&mut *tx)
    .await {
        return HttpResponse::InternalServerError().body("Failed to insert quiz");
    }

    let mut response_questions = Vec::new();

    for q in &req.questions {
        let question_id = Uuid::new_v4();
        if let Err(_) = sqlx::query!(
            "INSERT INTO questions (id, quiz_id, text) VALUES ($1, $2, $3)",
            question_id,
            quiz_id,
            q.text
        )
        .execute(&mut *tx)
        .await {
            return HttpResponse::InternalServerError().body("Failed to insert question");
        }

        let mut response_options = Vec::new();
        for o in &q.options {
            let option_id = Uuid::new_v4();
            if let Err(_) = sqlx::query!(
                "INSERT INTO question_options (id, question_id, text, is_correct) VALUES ($1, $2, $3, $4)",
                option_id,
                question_id,
                o.text,
                o.is_correct
            )
            .execute(&mut *tx)
            .await {
                return HttpResponse::InternalServerError().body("Failed to insert option");
            }
            response_options.push(QuestionOption {
                id: option_id,
                text: o.text.clone(),
                is_correct: o.is_correct,
            });
        }
        
        response_questions.push(Question {
            id: question_id,
            text: q.text.clone(),
            options: response_options,
        });
    }

    if let Err(_) = tx.commit().await {
        return HttpResponse::InternalServerError().body("Failed to commit transaction");
    }

    HttpResponse::Created().json(Quiz {
        id: quiz_id,
        title: req.title.clone(),
        category_id: req.category_id,
        questions: response_questions,
    })
}

#[utoipa::path(
    get,
    path = "/quizzes/{id}",
    params(
        ("id" = Uuid, Path, description = "Quiz ID")
    ),
    responses(
        (status = 200, description = "Get Quiz by ID", body = Quiz),
        (status = 404, description = "Quiz not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn get_quiz(
    data: web::Data<AppState>,
    path: web::Path<uuid::Uuid>,
) -> impl Responder {
    let quiz_id = path.into_inner();
    
    let quiz = match sqlx::query_as::<_, Quiz>(
        "SELECT id, title, category_id FROM quizzes WHERE id = $1"
    )
    .bind(quiz_id)
    .fetch_optional(&data.db)
    .await {
        Ok(Some(q)) => q,
        Ok(None) => return HttpResponse::NotFound().body("Quiz not found"),
        Err(_) => return HttpResponse::InternalServerError().body("Database error"),
    };

    let questions = match sqlx::query_as::<_, Question>(
        "SELECT id, text FROM questions WHERE quiz_id = $1"
    )
    .bind(quiz_id)
    .fetch_all(&data.db)
    .await {
        Ok(qs) => qs,
        Err(_) => return HttpResponse::InternalServerError().body("Database error fetching questions"),
    };

    let mut full_questions = Vec::new();
    for mut q in questions {
        let options = match sqlx::query_as::<_, QuestionOption>(
            "SELECT id, text, is_correct FROM question_options WHERE question_id = $1"
        )
        .bind(q.id)
        .fetch_all(&data.db)
        .await {
            Ok(opts) => opts,
            Err(_) => return HttpResponse::InternalServerError().body("Database error fetching options"),
        };
        q.options = options;
        full_questions.push(q);
    }

    let mut full_quiz = quiz;
    full_quiz.questions = full_questions;

    HttpResponse::Ok().json(full_quiz)
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct ListQuizzesFilter {
    category_id: Option<Uuid>,
}

#[utoipa::path(
    get,
    path = "/quizzes",
    params(
        ListQuizzesFilter
    ),
    responses(
        (status = 200, description = "List Quizzes", body = Vec<Quiz>),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn list_quizzes(data: web::Data<AppState>, filter: web::Query<ListQuizzesFilter>) -> impl Responder {
    let query_str = if let Some(cat_id) = filter.category_id {
        format!("SELECT id, title, category_id FROM quizzes WHERE category_id = '{}'", cat_id)
    } else {
        "SELECT id, title, category_id FROM quizzes".to_string()
    };

    let quizzes = match sqlx::query_as::<_, Quiz>(&query_str)
    .fetch_all(&data.db)
    .await {
        Ok(qs) => qs,
        Err(_) => return HttpResponse::InternalServerError().body("Database error"),
    };

    // Note: questions will be empty vec as initialized by default struct or we might need to be careful.
    // Actually, SQLx FromRow will try to fill fields. questions has #[sqlx(skip)].
    // So it will use Default implementation for Vec which is empty. 
    
    HttpResponse::Ok().json(quizzes)
}

#[utoipa::path(
    post,
    path = "/quizzes/{id}/solve",
    request_body = SubmitAnswerRequest,
    params(
        ("id" = Uuid, Path, description = "Quiz ID")
    ),
    responses(
        (status = 200, description = "Answer result", body = AnswerResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn submit_answer(
    data: web::Data<AppState>,
    _path: web::Path<uuid::Uuid>, // We don't strictly need quiz_id if we have question_id/option_id globally unique UUIDs, but usually good to validate hierarchy.
    req: web::Json<SubmitAnswerRequest>,
) -> impl Responder {
    // We can just check the option directly if UUIDs are unique global.
    let is_correct = match sqlx::query!(
        "SELECT is_correct FROM question_options WHERE id = $1 AND question_id = $2",
        req.option_id,
        req.question_id
    )
    .fetch_optional(&data.db)
    .await {
        Ok(Some(rec)) => rec.is_correct,
        Ok(None) => return HttpResponse::BadRequest().body("Invalid question or option"),
        Err(_) => return HttpResponse::InternalServerError().body("Database error"),
    };

    HttpResponse::Ok().json(AnswerResponse {
        correct: is_correct,
        message: if is_correct { "Correct!".to_string() } else { "Incorrect.".to_string() },
    })
}

#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "Registered successfully", body = String),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn auth_register(
    data: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
) -> impl Responder {
    let password_hash = match auth::hash_password(&req.password) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().body("Hashing failed"),
    };

    let user_id = Uuid::new_v4();

    if let Err(_) = sqlx::query!(
        "INSERT INTO users (id, username, password_hash) VALUES ($1, $2, $3)",
        user_id,
        req.username,
        password_hash
    )
    .execute(&data.db)
    .await {
        return HttpResponse::InternalServerError().body("Registration failed (username might be taken)");
    }

    HttpResponse::Ok().body("Registered successfully")
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Logged in successfully", body = TokenResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn auth_login(
    data: web::Data<AppState>,
    req: web::Json<LoginRequest>,
) -> impl Responder {
    let user = match sqlx::query_as!(
        User,
        "SELECT id, username, password_hash FROM users WHERE username = $1",
        req.username
    )
    .fetch_optional(&data.db)
    .await {
        Ok(Some(u)) => u,
        Ok(None) => return HttpResponse::Unauthorized().body("Invalid credentials"),
        Err(_) => return HttpResponse::InternalServerError().body("Database error"),
    };

    if !auth::verify_password(&user.password_hash, &req.password) {
        return HttpResponse::Unauthorized().body("Invalid credentials");
    }

    let token = match auth::create_token(&user.username) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().body("Token creation failed"),
    };

    HttpResponse::Ok().json(TokenResponse { token })
}

#[utoipa::path(
    post,
    path = "/categories",
    request_body = CreateCategoryRequest,
    responses(
        (status = 201, description = "Category created", body = Category),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn create_category(
    data: web::Data<AppState>,
    req: web::Json<CreateCategoryRequest>,
    _: auth::JwtMiddleware,
) -> impl Responder {
    let category_id = Uuid::new_v4();
    if let Err(_) = sqlx::query!(
        "INSERT INTO categories (id, name) VALUES ($1, $2)",
        category_id,
        req.name
    )
    .execute(&data.db)
    .await {
        return HttpResponse::InternalServerError().body("Failed to create category");
    }
    HttpResponse::Created().json(Category { id: category_id, name: req.name.clone() })
}

#[utoipa::path(
    get,
    path = "/categories",
    responses(
        (status = 200, description = "List Categories", body = Vec<Category>),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn list_categories(data: web::Data<AppState>) -> impl Responder {
    let categories = match sqlx::query_as!(
        Category,
        "SELECT id, name FROM categories"
    )
    .fetch_all(&data.db)
    .await {
        Ok(cats) => cats,
        Err(_) => return HttpResponse::InternalServerError().body("Database error"),
    };
    HttpResponse::Ok().json(categories)
}
