use std::future::Future;
use std::net::SocketAddr;

use serde_json::json;
use tempfile::NamedTempFile;
use tokio::net::TcpListener;
use visdom_harness::error::AppError;
use visdom_harness::llm::{InferenceMessage, InferenceResult, LlmClient, ToolCallRecord, ToolSpec};
use visdom_harness::{AppState, db};

#[derive(Clone)]
struct MockLlmClient;

impl LlmClient for MockLlmClient {
    fn infer(
        &self,
        _system_prompt: &str,
        messages: &[InferenceMessage],
        _tools: &[ToolSpec],
    ) -> impl Future<Output = Result<InferenceResult, AppError>> + Send {
        let prompt_text = messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default();
        async move {
            Ok(InferenceResult {
                prompt_text,
                response_text: String::new(),
                tool_calls: vec![ToolCallRecord {
                    id: "tc-001".to_string(),
                    name: "get_weather".to_string(),
                    arguments: json!({ "city": "London" }),
                }],
            })
        }
    }
}

async fn spawn_app() -> SocketAddr {
    let db_file = NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}?mode=rwc", db_file.path().display());
    let pool = db::connect_and_migrate(&db_url).await.unwrap();
    std::mem::forget(db_file);

    let state = AppState {
        pool,
        llm: MockLlmClient,
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = visdom_harness::build_app(state);
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

#[tokio::test]
async fn post_infer_returns_id_and_get_returns_record() {
    let addr = spawn_app().await;
    let client = reqwest::Client::new();

    // POST /debug/infer
    let resp = client
        .post(format!("http://{addr}/debug/infer"))
        .json(&json!({ "prompt": "What is the weather in London?" }))
        .send()
        .await
        .expect("POST /debug/infer failed");

    assert_eq!(resp.status(), 200, "expected 200 from POST /debug/infer");

    let body: serde_json::Value = resp.json().await.expect("body not JSON");
    let id = body["id"].as_str().expect("missing id field").to_string();
    assert!(!id.is_empty(), "id should not be empty");

    // GET /debug/inferences/:id
    let resp = client
        .get(format!("http://{addr}/debug/inferences/{id}"))
        .send()
        .await
        .expect("GET /debug/inferences/:id failed");

    assert_eq!(
        resp.status(),
        200,
        "expected 200 from GET /debug/inferences/:id"
    );

    let record: serde_json::Value = resp.json().await.expect("record not JSON");
    assert_eq!(record["id"], id);

    let tool_calls: serde_json::Value =
        serde_json::from_str(record["tool_calls_json"].as_str().unwrap()).unwrap();
    assert_eq!(tool_calls[0]["name"], "get_weather");
    assert_eq!(tool_calls[0]["arguments"]["city"], "London");
}

#[tokio::test]
async fn post_infer_empty_prompt_returns_400() {
    let addr = spawn_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("http://{addr}/debug/infer"))
        .json(&json!({ "prompt": "" }))
        .send()
        .await
        .expect("POST /debug/infer failed");

    assert_eq!(resp.status(), 400, "expected 400 for empty prompt");

    let body: serde_json::Value = resp.json().await.expect("body not JSON");
    assert!(
        body["error"].as_str().unwrap_or("").contains("prompt"),
        "error message should mention prompt"
    );
}

#[tokio::test]
async fn get_inference_unknown_id_returns_404() {
    let addr = spawn_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("http://{addr}/debug/inferences/nonexistent-id"))
        .send()
        .await
        .expect("GET /debug/inferences/:id failed");

    assert_eq!(resp.status(), 404, "expected 404 for unknown id");
}

#[tokio::test]
async fn list_inferences_returns_all_records() {
    let addr = spawn_app().await;
    let client = reqwest::Client::new();

    // Initially empty
    let resp = client
        .get(format!("http://{addr}/debug/inferences"))
        .send()
        .await
        .expect("GET /debug/inferences failed");
    assert_eq!(resp.status(), 200);
    let records: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(records.as_array().unwrap().len(), 0);

    // After one POST
    client
        .post(format!("http://{addr}/debug/infer"))
        .json(&json!({ "prompt": "test prompt" }))
        .send()
        .await
        .unwrap();

    let resp = client
        .get(format!("http://{addr}/debug/inferences"))
        .send()
        .await
        .expect("GET /debug/inferences failed");
    assert_eq!(resp.status(), 200);
    let records: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(records.as_array().unwrap().len(), 1);
}
