use sqlx::postgres::PgPoolOptions;
use dotenv::dotenv;
use std::env;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .connect(&database_url)
        .await?;

    println!("Resetting database...");

    // Truncate tables but keep schema
    sqlx::query!("TRUNCATE TABLE quiz_tags, question_options, questions, quizzes, categories, tags, users RESTART IDENTITY CASCADE")
        .execute(&pool)
        .await?;

    println!("Database reset successfully!");
    Ok(())
}
