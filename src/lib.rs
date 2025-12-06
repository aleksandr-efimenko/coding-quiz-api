use actix_web::{web, App, HttpServer, middleware};
use actix_web::dev::Server;
use std::net::TcpListener;
use env_logger::Env;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use sqlx::PgPool;

pub mod models;
pub mod state;
pub mod handlers;
pub mod auth;

use state::AppState;
use handlers::{health_check, create_quiz, get_quiz, list_quizzes, submit_answer, delete_quiz, update_quiz};
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
pub struct ApiDoc;

pub struct SecurityAddon;

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

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let data = web::Data::new(AppState {
        db: db_pool,
    });

    // Logging init usually handled in main, but we can check if it's already set
    // env_logger::init_from_env(Env::default().default_filter_or("info"));

    let server = HttpServer::new(move || {
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
    .listen(listener)?
    .run();

    Ok(server)
}
