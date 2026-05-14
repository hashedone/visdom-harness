use std::net::SocketAddr;
use tokio::net::TcpListener;

async fn spawn_app() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = visdom_harness::build_app();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

#[tokio::test]
async fn health_returns_200_with_ok_body() {
    let addr = spawn_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("http://{addr}/health"))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("body not JSON");
    assert_eq!(body["status"], "ok");
}
