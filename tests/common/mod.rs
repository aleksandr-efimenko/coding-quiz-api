use coding_quiz_api::run;
use std::net::TcpListener;

#[allow(dead_code)]
pub struct TestApp {
    pub address: String,
    pub api_client: reqwest::Client,
}

pub async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let server = run(listener, vec![], vec![]).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp {
        address,
        api_client: reqwest::Client::new(),
    }
}
