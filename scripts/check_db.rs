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

    // Check constraints
    let constraints: Vec<(String, String, String)> = sqlx::query_as(
        r#"
        SELECT tc.constraint_name::text, tc.table_name::text, kcu.column_name::text
        FROM information_schema.table_constraints AS tc
        JOIN information_schema.key_column_usage AS kcu
          ON tc.constraint_name = kcu.constraint_name
          AND tc.table_schema = kcu.table_schema
        WHERE tc.constraint_type = 'FOREIGN KEY'
          AND tc.table_name IN ('questions', 'question_options')
        "#
    )
    .fetch_all(&pool)
    .await?;
    
    println!("Constraints:");
    for (name, table, col) in constraints {
        println!("Table: {}, Column: {}, Constraint: {}", table, col, name);
    }

    Ok(())
}
