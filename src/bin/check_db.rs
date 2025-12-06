use sqlx::postgres::PgPoolOptions;
use dotenv::dotenv;
use std::env;

#[derive(sqlx::FromRow, Debug)]
struct Tag {
    id: uuid::Uuid, 
    name: String,
}

#[derive(sqlx::FromRow, Debug)]
struct QuizTag {
    quiz_id: uuid::Uuid,
    tag_id: uuid::Uuid,
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .connect(&database_url)
        .await?;

    let tags: Vec<Tag> = sqlx::query_as("SELECT id, name FROM tags")
        .fetch_all(&pool)
        .await?;
    println!("Tags count: {}", tags.len());
    for t in tags {
        println!("Tag: {:?}", t);
    }

    let quiz_tags: Vec<QuizTag> = sqlx::query_as("SELECT quiz_id, tag_id FROM quiz_tags")
        .fetch_all(&pool)
        .await?;
    println!("QuizTags count: {}", quiz_tags.len());
    for qt in quiz_tags {
        println!("QuizTag: {:?}", qt);
    }

    Ok(())
}
