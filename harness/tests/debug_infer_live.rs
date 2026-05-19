use std::net::SocketAddr;

use serde_json::json;
use tokio::net::TcpListener;
use visdom_harness::llm::anthropic::AnthropicLlmClient;
use visdom_harness::{AppState, db};

async fn spawn_live_app(model: &str) -> SocketAddr {
    let pool = db::in_memory_pool().await.unwrap();
    let llm = AnthropicLlmClient::from_env(model).expect("ANTHROPIC_API_KEY must be set");
    let state = AppState { pool, llm, integrations: visdom_harness::IntegrationRegistry::new() };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = visdom_harness::build_app(state);
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

/// Live round-trip against Anthropic's API.
///
/// Run with:
///   ANTHROPIC_API_KEY=... cargo test --test debug_infer_live -- --ignored --nocapture
#[tokio::test]
#[ignore]
async fn live_anthropic_round_trip() {
    dotenvy::dotenv().ok();

    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        eprintln!("skipping: ANTHROPIC_API_KEY not set");
        return;
    }

    let model =
        std::env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "claude-sonnet-4-6".to_string());
    let addr = spawn_live_app(&model).await;
    let client = reqwest::Client::new();

    // POST /debug/infer with a prompt that should elicit a tool call
    let resp = client
        .post(format!("http://{addr}/debug/infer"))
        .json(&json!({ "prompt": "What is the weather in Paris? Use the get_weather tool." }))
        .send()
        .await
        .expect("POST /debug/infer failed");

    let status = resp.status();
    let body: serde_json::Value = resp.json().await.expect("body not JSON");
    assert_eq!(
        status.as_u16(),
        200,
        "expected 200 from POST /debug/infer, got {status}: {body}"
    );
    let id = body["id"].as_str().expect("missing id field").to_string();
    assert!(!id.is_empty(), "id should not be empty");
    eprintln!("inference id: {id}");

    // GET /debug/inferences/{id} — verify persisted record
    let resp = client
        .get(format!("http://{addr}/debug/inferences/{id}"))
        .send()
        .await
        .expect("GET /debug/inferences/:id failed");

    let status = resp.status();
    let record: serde_json::Value = resp.json().await.expect("record not JSON");
    assert_eq!(
        status.as_u16(),
        200,
        "expected 200 from GET /debug/inferences/:id, got {status}: {record}"
    );

    let response_text = record["response_text"].as_str().unwrap_or("");
    eprintln!("response_text: {response_text:?}");

    let tool_calls: serde_json::Value =
        serde_json::from_str(record["tool_calls_json"].as_str().unwrap())
            .expect("tool_calls_json must be valid JSON");

    let arr = tool_calls
        .as_array()
        .expect("tool_calls_json must be an array");
    assert!(!arr.is_empty(), "expected at least one tool call");

    let get_weather_call = arr.iter().find(|tc| tc["name"] == "get_weather");
    assert!(
        get_weather_call.is_some(),
        "expected a tool call named 'get_weather', got: {tool_calls}"
    );

    eprintln!("tool calls: {tool_calls}");
}
