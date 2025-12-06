use actix_web::{web, HttpResponse, Responder};
use crate::models::{
    CreateQuizRequest, Quiz, Question, QuestionOption, 
    SubmitAnswerRequest, AnswerResponse,
    RegisterRequest, LoginRequest, TokenResponse, User,
    Category, CreateCategoryRequest, UpdateQuizRequest, Tag
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
            "INSERT INTO questions (id, quiz_id, text, explanation) VALUES ($1, $2, $3, $4)",
            question_id,
            quiz_id,
            q.text,
            q.explanation
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
            explanation: q.explanation.clone(),
        });
    }

    if let Some(tags) = &req.tags {
        for tag_name in tags {
            // Upsert tag
            let tag_id = match sqlx::query!(
                "INSERT INTO tags (id, name) VALUES ($1, $2) ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name RETURNING id",
                Uuid::new_v4(),
                tag_name
            )
            .fetch_one(&mut *tx)
            .await {
                Ok(rec) => rec.id,
                Err(_) => return HttpResponse::InternalServerError().body("Failed to upsert tag"),
            };

            // Link quiz to tag
            if let Err(_) = sqlx::query!(
                "INSERT INTO quiz_tags (quiz_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                quiz_id,
                tag_id
            )
            .execute(&mut *tx)
            .await {
                return HttpResponse::InternalServerError().body("Failed to link tag");
            }
        }
    }

    if let Err(_) = tx.commit().await {
        return HttpResponse::InternalServerError().body("Failed to commit transaction");
    }

    // Since we just committed, we can create the response object directly (or re-fetch, but direct is faster)
    HttpResponse::Created().json(Quiz {
        id: quiz_id,
        title: req.title.clone(),
        category_id: req.category_id,
        questions: response_questions,
        tags: req.tags.clone().unwrap_or_default(),
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
    
    let rec = match sqlx::query!(
        "SELECT id, title, category_id FROM quizzes WHERE id = $1",
        quiz_id
    )
    .fetch_optional(&data.db)
    .await {
        Ok(Some(q)) => q,
        Ok(None) => return HttpResponse::NotFound().body("Quiz not found"),
        Err(_) => return HttpResponse::InternalServerError().body("Database error fetching quiz"),
    };

    let mut full_quiz = Quiz {
        id: rec.id,
        title: rec.title,
        category_id: rec.category_id,
        questions: vec![],
        tags: vec![],
    };

    let questions = match sqlx::query_as::<_, Question>(
        "SELECT id, text, explanation FROM questions WHERE quiz_id = $1"
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

    full_quiz.questions = full_questions;

    // Fetch tags
    let tags = match sqlx::query!(
        "SELECT t.name FROM tags t JOIN quiz_tags qt ON t.id = qt.tag_id WHERE qt.quiz_id = $1",
        quiz_id
    )
    .fetch_all(&data.db)
    .await {
        Ok(recs) => recs.into_iter().map(|r| r.name).collect(),
        Err(_) => return HttpResponse::InternalServerError().body("Database error fetching tags"),
    };
    full_quiz.tags = tags;

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
    let where_clause = if let Some(cat_id) = filter.category_id {
        format!("WHERE q.category_id = '{}'", cat_id)
    } else {
        "".to_string()
    };

    let query_str = format!(
        r#"
        SELECT 
            q.id, q.title, q.category_id, 
            COALESCE(array_agg(t.name) FILTER (WHERE t.name IS NOT NULL), '{{}}') as tags
        FROM quizzes q
        LEFT JOIN quiz_tags qt ON q.id = qt.quiz_id
        LEFT JOIN tags t ON qt.tag_id = t.id
        {}
        GROUP BY q.id
        "#,
        where_clause
    );

    let quizzes = match sqlx::query_as::<_, Quiz>(&query_str)
    .fetch_all(&data.db)
    .await {
        Ok(qs) => qs,
        Err(e) => {
            log::error!("List quizzes error: {}", e);
            return HttpResponse::InternalServerError().body("Database error");
        }
    };
    
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

    // Fetch explanation if needed, but usually we want it regardless of correct/incorrect to learn
    // Or maybe only on correct? The user prompt said "Make them helpful". Showing explanation on incorrect is very helpful.
    let question = match sqlx::query!(
        "SELECT explanation FROM questions WHERE id = $1",
        req.question_id
    )
    .fetch_optional(&data.db)
    .await {
        Ok(Some(q)) => q,
        _ => return HttpResponse::InternalServerError().body("Database error fetching question"),
    };

    HttpResponse::Ok().json(AnswerResponse {
        correct: is_correct,
        message: if is_correct { "Correct!".to_string() } else { "Incorrect.".to_string() },
        explanation: question.explanation,
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

#[utoipa::path(
    delete,
    path = "/quizzes/{id}",
    params(
        ("id" = Uuid, Path, description = "Quiz ID")
    ),
    responses(
        (status = 204, description = "Quiz deleted"),
        (status = 404, description = "Quiz not found"),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn delete_quiz(
    data: web::Data<AppState>,
    path: web::Path<Uuid>,
    _: auth::JwtMiddleware,
) -> impl Responder {
    let quiz_id = path.into_inner();
    let result = sqlx::query!("DELETE FROM quizzes WHERE id = $1", quiz_id)
        .execute(&data.db)
        .await;

    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                HttpResponse::NotFound().body("Quiz not found")
            } else {
                HttpResponse::NoContent().finish()
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Database error"),
    }
}

#[utoipa::path(
    put,
    path = "/quizzes/{id}",
    request_body = UpdateQuizRequest,
    params(
        ("id" = Uuid, Path, description = "Quiz ID")
    ),
    responses(
        (status = 200, description = "Quiz updated", body = Quiz),
        (status = 404, description = "Quiz not found"),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn update_quiz(
    data: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateQuizRequest>,
    _: auth::JwtMiddleware,
) -> impl Responder {
    let quiz_id = path.into_inner();
    let mut tx = match data.db.begin().await {
        Ok(tx) => tx,
        Err(_) => return HttpResponse::InternalServerError().body("Database error starting transaction")
    };

    let mut exists = false;
    let check = sqlx::query!("SELECT id FROM quizzes WHERE id = $1", quiz_id).fetch_optional(&mut *tx).await;
    match check {
        Ok(Some(_)) => exists = true,
        Ok(None) => return HttpResponse::NotFound().body("Quiz not found"),
        Err(_) => return HttpResponse::InternalServerError().body("Database error checking quiz"),
    }

    if let Some(title) = &req.title {
        if let Err(_) = sqlx::query!("UPDATE quizzes SET title = $1 WHERE id = $2", title, quiz_id)
            .execute(&mut *tx).await {
             return HttpResponse::InternalServerError().body("Error updating title");
        }
    }
    
    if let Some(category_id) = req.category_id {
         if let Err(_) = sqlx::query!("UPDATE quizzes SET category_id = $1 WHERE id = $2", category_id, quiz_id)
            .execute(&mut *tx).await {
             return HttpResponse::InternalServerError().body("Error updating category");
        }
    }

    // Handle Tags if provided
    if let Some(tags) = &req.tags {
        // Clear existing tags
        if let Err(_) = sqlx::query!("DELETE FROM quiz_tags WHERE quiz_id = $1", quiz_id)
            .execute(&mut *tx).await {
            return HttpResponse::InternalServerError().body("Error clearing tags");
        }

        // Add new tags
        for tag_name in tags {
             let tag = sqlx::query_as!(
                Tag,
                r#"
                WITH inserted_tag AS (
                    INSERT INTO tags (id, name)
                    VALUES ($1, $2)
                    ON CONFLICT (name) DO NOTHING
                    RETURNING id, name
                )
                SELECT id AS "id!", name AS "name!" FROM inserted_tag
                UNION ALL
                SELECT id AS "id!", name AS "name!" FROM tags WHERE name = $2
                "#,
                Uuid::new_v4(),
                tag_name
            )
            .fetch_one(&mut *tx)
            .await;

            match tag {
                Ok(t) => {
                    if let Err(_) = sqlx::query!(
                        "INSERT INTO quiz_tags (quiz_id, tag_id) VALUES ($1, $2)",
                        quiz_id,
                        t.id
                    )
                    .execute(&mut *tx)
                    .await {
                         return HttpResponse::InternalServerError().body("Error linking tag");
                    }
                },
                Err(_) => return HttpResponse::InternalServerError().body("Error upserting tag"),
            }
        }
    }
    
    if let Err(_) = tx.commit().await {
        return HttpResponse::InternalServerError().body("Database error committing transaction");
    }

    // Re-fetch quiz logic
    let rec = match sqlx::query!(
        "SELECT id, title, category_id FROM quizzes WHERE id = $1",
        quiz_id
    )
    .fetch_optional(&data.db)
    .await {
        Ok(Some(q)) => q,
        Ok(None) => return HttpResponse::NotFound().body("Quiz not found"),
        Err(_) => return HttpResponse::InternalServerError().body("Database error fetching quiz"),
    };

    let questions = match sqlx::query_as::<_, Question>(
        "SELECT id, text, explanation FROM questions WHERE quiz_id = $1"
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
    
    let mut full_quiz = Quiz {
        id: rec.id,
        title: rec.title,
        category_id: rec.category_id,
        questions: full_questions,
        tags: vec![],
    };

    // Fetch tags
    let tags = match sqlx::query!(
        "SELECT t.name FROM tags t JOIN quiz_tags qt ON t.id = qt.tag_id WHERE qt.quiz_id = $1",
        quiz_id
    )
    .fetch_all(&data.db)
    .await {
        Ok(recs) => recs.into_iter().map(|r| r.name).collect(),
        Err(_) => return HttpResponse::InternalServerError().body("Database error fetching tags"),
    };
    full_quiz.tags = tags;

    HttpResponse::Ok().json(full_quiz)
}
