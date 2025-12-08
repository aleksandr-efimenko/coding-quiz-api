use actix_web::{web, App, HttpServer, middleware};
use actix_web::dev::Server;
use std::net::TcpListener;
use std::sync::RwLock;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::models::{
    CreateQuizRequest, Quiz, Question, QuestionOption, 
    SubmitAnswerRequest, AnswerResponse,
    Category, CreateCategoryRequest, CreateQuestionRequest, CreateOptionRequest,
    UpdateQuizRequest, PaginationParams, ErrorResponse
};

pub mod models;
pub mod state;
pub mod handlers;
pub mod auth; // Empty module
pub mod id;

use state::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health_check,
        handlers::create_category,
        handlers::list_categories,
        handlers::create_quiz,
        handlers::get_quiz,
        handlers::list_quizzes,
        handlers::submit_answer,
        handlers::delete_quiz,
        handlers::update_quiz,
        handlers::get_random_quiz,
    ),
    components(
        schemas(
            CreateQuizRequest, Quiz, Question, QuestionOption, 
            SubmitAnswerRequest, AnswerResponse,
            Category, CreateCategoryRequest, CreateQuestionRequest, CreateOptionRequest,
            UpdateQuizRequest,
            PaginationParams, ErrorResponse
        )
    ),
    tags(
        (name = "System", description = "System endpoints"),
        (name = "Management", description = "Quiz management endpoints"),
        (name = "Consumption", description = "Public consumption endpoints")
    )
)]
pub struct ApiDoc;

pub fn run(listener: TcpListener, quizzes: Vec<Quiz>, categories: Vec<Category>) -> Result<Server, std::io::Error> {
    let data = web::Data::new(AppState {
        quizzes: RwLock::new(quizzes),
        categories: RwLock::new(categories),
    });

    let server = HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::Logger::default())
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi())
            )
            .route("/health", web::get().to(handlers::health_check))
            .service(
                web::scope("/categories")
                    .route("", web::post().to(handlers::create_category))
                    .route("", web::get().to(handlers::list_categories))
            )
            .service(
                web::scope("/quizzes")
                    .route("", web::post().to(handlers::create_quiz))
                    .route("", web::get().to(handlers::list_quizzes))
                    .route("/random", web::get().to(handlers::get_random_quiz))
                    .route("/{id}", web::get().to(handlers::get_quiz))
                    .route("/{id}", web::put().to(handlers::update_quiz))
                    .route("/{id}", web::delete().to(handlers::delete_quiz))
                    .route("/{id}/solve", web::post().to(handlers::submit_answer))
            )
    })
    .listen(listener)?
    .run();

    Ok(server)
}
