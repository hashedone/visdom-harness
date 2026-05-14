use std::net::SocketAddr;
use tempfile::NamedTempFile;
use tokio::net::TcpListener;
use visdom_harness::{AppState, db};

async fn spawn_app() -> SocketAddr {
    let db_file = NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}?mode=rwc", db_file.path().display());

    let pool = db::connect_and_migrate(&db_url).await.unwrap();
    // Keep the tempfile alive for the duration of the test by leaking it.
    // The OS will clean it up when the process exits.
    std::mem::forget(db_file);

    let state = AppState { pool };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = visdom_harness::build_app(state);
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
