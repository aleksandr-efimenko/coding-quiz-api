use coding_quiz_api::run;
use sqlx::PgPool;
use std::net::TcpListener;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[allow(dead_code)]
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub api_client: reqwest::Client,
}

pub async fn spawn_app() -> TestApp {
    dotenv::dotenv().ok();
    // Randomize database
    let pool = configure_database().await;

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

async fn configure_database() -> PgPool {
    let connection = PgPoolOptions::new()
        .connect_with(
            std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set")
                .parse::<sqlx::postgres::PgConnectOptions>()
                .expect("Failed to parse DATABASE_URL")
                .database("postgres")
        )
        .await
        .expect("Failed to connect to Postgres");

    let database_name = format!("test_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));
    
    // Create query
    sqlx::query(&format!("CREATE DATABASE \"{}\"", database_name))
        .execute(&connection)
        .await
        .expect("Failed to create database");

    // Migrate
    let pool = PgPoolOptions::new()
        .connect_with(
            std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set")
                .parse::<sqlx::postgres::PgConnectOptions>()
                .expect("Failed to parse DATABASE_URL")
                .database(&database_name)
        )
        .await
        .expect("Failed to connect to new database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate database");

    pool
}

pub async fn get_auth_token(app: &TestApp) -> String {
    let username = format!("user_{}", Uuid::new_v4());
    let password = "password123";

    let register_body = serde_json::json!({
        "username": username,
        "password": password
    });

    app.api_client
        .post(&format!("{}/auth/register", &app.address))
        .json(&register_body)
        .send()
        .await
        .expect("Failed to execute request.");

    let login_body = serde_json::json!({
        "username": username,
        "password": password
    });

    let response = app.api_client
        .post(&format!("{}/auth/login", &app.address))
        .json(&login_body)
        .send()
        .await
        .expect("Failed to execute request.");
        
    let json: serde_json::Value = response.json().await.expect("Failed to read JSON");
    json["token"].as_str().unwrap().to_string()
}

pub async fn get_api_key(app: &TestApp, token: &str) -> String {
    let response = app.api_client
        .post(&format!("{}/auth/api-keys", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request.");
        
    // 201 Created
    assert_eq!(201, response.status().as_u16());
    let json: serde_json::Value = response.json().await.expect("Failed to read JSON");
    json["api_key"].as_str().unwrap().to_string()
}
