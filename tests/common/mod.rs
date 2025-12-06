use coding_quiz_api::run;
use sqlx::PgPool;
use std::net::TcpListener;
use sqlx::postgres::PgPoolOptions;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub api_client: reqwest::Client,
}

pub async fn spawn_app() -> TestApp {
    // Load .env just in case, though tests might use a different logic
    dotenv::dotenv().ok();

    // In a real scenario, we'd randomize database name here. 
    // For simplicity, we'll reuse the same DB but perhaps we should truncate?
    // Let's assume the user is okay with using the existing DB for now as creating dynamic DBs requires more setup.
    // To be safe, we will just connect to the default DB. 
    // WARNING: This means tests might interfere with local data or each other if not careful.
    // A robust solution involves checking `DATABASE_URL`, parsing it, changing dbname, creating it.
    
    // For this task, we will just connect to the configured DATABASE_URL.
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to pool");

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let server = run(listener, pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: pool,
        api_client: reqwest::Client::new(),
    }
}
