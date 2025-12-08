use coding_quiz_api::id::Id;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug)]
struct QuizSeed {
    title: String,
    tags: Vec<String>,
    questions: Vec<QuestionSeed>,
}

#[derive(Deserialize, Debug)]
struct QuestionSeed {
    text: String,
    explanation: String,
    options: Vec<OptionSeed>,
}

#[derive(Deserialize, Debug)]
struct OptionSeed {
    text: String,
    is_correct: bool,
    description: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    // Ensure DATABASE_URL is set
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    println!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let seed_dir = Path::new("seed/javascript");
    if !seed_dir.exists() {
        println!("Seed directory not found: {:?}", seed_dir);
        return Ok(());
    }

    let mut entries = fs::read_dir(seed_dir)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    // Sort for deterministic order
    entries.sort();

    for path in entries {
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            println!("Seeding from file: {:?}", path);
            let content = fs::read_to_string(&path)?;
            let quiz_seed: QuizSeed = serde_json::from_str(&content)?;
            
            insert_quiz(&pool, quiz_seed).await?;
        }
    }

    println!("Seeding completed successfully.");
    Ok(())
}

async fn insert_quiz(pool: &sqlx::PgPool, quiz: QuizSeed) -> Result<(), Box<dyn std::error::Error>> {
    let mut tx = pool.begin().await?;

    // 1. Create Quiz
    let quiz_id = Id::new();
    let category_id: Option<Id> = None; // Default, simplistic for now
    
    sqlx::query!(
        "INSERT INTO quizzes (id, title, category_id) VALUES ($1, $2, $3)",
        quiz_id.to_i64(),
        quiz.title,
        category_id.map(|v| v.to_i64())
    )
    .execute(&mut *tx)
    .await?;

    // 2. Handle Tags
    for tag_name in quiz.tags {
        // Upsert tag
        let tag_id = match sqlx::query!(
            "INSERT INTO tags (id, name) VALUES ($1, $2) ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name RETURNING id",
            Id::new().to_i64(),
            tag_name
        )
        .fetch_one(&mut *tx)
        .await {
            Ok(rec) => Id::from(rec.id),
            Err(e) => {
                println!("Error inserting tag {}: {}", tag_name, e);
                return Err(e.into());
            }
        };

        // Link tag
        sqlx::query!(
            "INSERT INTO quiz_tags (quiz_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            quiz_id.to_i64(),
            tag_id.to_i64()
        )
        .execute(&mut *tx)
        .await?;
    }

    // 3. Create Questions
    for q in quiz.questions {
        let question_id = Id::new();
        let result = sqlx::query!(
            "INSERT INTO questions (id, quiz_id, text, explanation) VALUES ($1, $2, $3, $4) ON CONFLICT (text) DO NOTHING",
            question_id.to_i64(),
            quiz_id.to_i64(),
            q.text,
            q.explanation
        )
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            println!("Skipping duplicate question: {}", q.text);
            continue;
        }

        // 4. Create Options
        for opt in q.options {
            let option_id = Id::new();
            sqlx::query!(
                "INSERT INTO question_options (id, question_id, text, is_correct, description) VALUES ($1, $2, $3, $4, $5)",
                option_id.to_i64(),
                question_id.to_i64(),
                opt.text,
                opt.is_correct,
                opt.description
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}
