use coding_quiz_api::run;
use coding_quiz_api::models::{Quiz, Question, QuestionOption};
use coding_quiz_api::id::Id;
use std::net::TcpListener;
use env_logger::Env;
use walkdir::WalkDir;
use serde::Deserialize;

#[derive(Deserialize)]
struct QuizSeed {
    title: String,
    tags: Option<Vec<String>>,
    questions: Vec<QuestionSeed>,
}

#[derive(Deserialize)]
struct QuestionSeed {
    text: String,
    options: Vec<OptionSeed>,
    explanation: Option<String>,
}

#[derive(Deserialize)]
struct OptionSeed {
    text: String,
    is_correct: bool,
    description: Option<String>,
}

fn load_quizzes() -> Vec<Quiz> {
    let mut quizzes = Vec::new();
    let seed_dir = "seed/javascript";
    
    log::info!("Loading quizzes from {}", seed_dir);

    for entry in WalkDir::new(seed_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.path().extension().map_or(false, |ext| ext == "json") {
            let path = entry.path();
            log::info!("Loading file: {:?}", path);
            
            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    log::error!("Failed to read file {:?}: {}", path, e);
                    continue;
                }
            };
            
            let seed: QuizSeed = match serde_json::from_str(&content) {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Failed to parse JSON {:?}: {}", path, e);
                    continue;
                }
            };

            // Map to Domain Models with NEW IDs every boot
            let quiz_id = Id::new();
            let questions: Vec<Question> = seed.questions.into_iter().map(|q| {
                let q_id = Id::new();
                let options = q.options.into_iter().map(|o| {
                    QuestionOption {
                        id: Id::new(),
                        text: o.text,
                        is_correct: o.is_correct,
                        description: o.description,
                    }
                }).collect();
                Question {
                    id: q_id,
                    text: q.text,
                    options,
                    explanation: q.explanation,
                }
            }).collect();

            let tags = seed.tags.unwrap_or_default();
            // Assuming category is implied by tags or null for now as seeds don't have category_id?
            // Actually metadata says "categories" or derived. Let's just create a category if needed or leave null.
            // Leaving category_id None for simplicity as seeds don't have it directly mapped to specific ID.
            
            quizzes.push(Quiz {
                id: quiz_id,
                title: seed.title,
                category_id: None, 
                questions,
                tags,
            });
        }
    }
    log::info!("Loaded {} quizzes", quizzes.len());
    quizzes
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let quizzes = load_quizzes();

    log::info!("Starting server at http://0.0.0.0:8080");
    log::info!("Swagger UI available at http://localhost:8080/swagger-ui/");

    let listener = TcpListener::bind("0.0.0.0:8080")?;
    run(listener, quizzes, Vec::new())?.await
}
