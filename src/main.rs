use actix_web::{web, App, HttpServer, middleware};
use env_logger::Env;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod models;
mod state;
mod handlers;
mod auth;

use state::AppState;
use handlers::{health_check, create_quiz, get_quiz, list_quizzes, submit_answer, auth_register, auth_login};
use models::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health_check,
        handlers::auth_register,
        handlers::auth_login,
        handlers::create_category,
        handlers::list_categories,
        handlers::create_quiz,
        handlers::get_quiz,
        handlers::list_quizzes,
        handlers::submit_answer,
        handlers::delete_quiz,
        handlers::update_quiz,
    ),
    components(
        schemas(
            CreateQuizRequest, Quiz, Question, QuestionOption, 
            SubmitAnswerRequest, AnswerResponse,
            RegisterRequest, LoginRequest, TokenResponse, 
            Category, CreateCategoryRequest, CreateQuestionRequest, CreateOptionRequest,
            UpdateQuizRequest
        )
    ),
    tags(
        (name = "coding-quiz-api", description = "Coding Quiz API")
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "jwt",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let data = web::Data::new(AppState {
        db: pool,
    });

    log::info!("Starting server at http://127.0.0.1:8080");
    log::info!("Swagger UI available at http://127.0.0.1:8080/swagger-ui/");

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::Logger::default())
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi())
            )
            .route("/health", web::get().to(health_check))
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(handlers::auth_register))
                    .route("/login", web::post().to(handlers::auth_login))
            )
            .service(
                web::scope("/categories")
                    .route("", web::post().to(handlers::create_category))
                    .route("", web::get().to(handlers::list_categories))
            )
            .service(
                web::scope("/quizzes")
                    .route("", web::post().to(create_quiz))
                    .route("", web::get().to(list_quizzes))
                    .route("/{id}", web::get().to(get_quiz))
                    .route("/{id}", web::put().to(handlers::update_quiz))
                    .route("/{id}", web::delete().to(handlers::delete_quiz))
                    .route("/{id}/solve", web::post().to(submit_answer))
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
