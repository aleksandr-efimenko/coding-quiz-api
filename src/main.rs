use coding_quiz_api::run;
use std::net::TcpListener;
use env_logger::Env;

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

    log::info!("Starting server at http://0.0.0.0:8080");
    log::info!("Swagger UI available at http://localhost:8080/swagger-ui/");

    let listener = TcpListener::bind("0.0.0.0:8080")?;
    run(listener, pool)?.await
}
