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

    // Drop schema to fully wipe everything including types, tables, and migration history
    sqlx::query!("DROP SCHEMA public CASCADE").execute(&pool).await?;
    sqlx::query!("CREATE SCHEMA public").execute(&pool).await?;
    sqlx::query!("GRANT ALL ON SCHEMA public TO public").execute(&pool).await?;

    println!("Database reset successfully!");
    Ok(())
}
