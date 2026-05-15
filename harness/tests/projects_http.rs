use std::net::SocketAddr;

use serde_json::json;
use tokio::net::TcpListener;
use visdom_harness::error::AppError;
use visdom_harness::llm::{InferenceMessage, InferenceResult, LlmClient, ToolCallRecord, ToolSpec};
use visdom_harness::{AppState, db};

#[derive(Clone)]
struct MockLlmClient;

impl LlmClient for MockLlmClient {
    async fn infer(
        &self,
        _system_prompt: &str,
        _messages: &[InferenceMessage],
        _tools: &[ToolSpec],
    ) -> Result<InferenceResult, AppError> {
        Ok(InferenceResult {
            prompt_text: String::new(),
            response_text: String::new(),
            tool_calls: vec![ToolCallRecord {
                id: "tc-001".to_string(),
                name: "noop".to_string(),
                arguments: json!({}),
            }],
        })
    }
}

async fn spawn_app() -> SocketAddr {
    let pool = db::in_memory_pool().await.unwrap();
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
async fn create_project_returns_201_with_project() {
    let addr = spawn_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("http://{addr}/projects"))
        .json(&json!({ "name": "Alpha", "description": "first project" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    let id = body["id"].as_str().expect("missing id");
    assert!(!id.is_empty());
    assert_eq!(body["name"], "Alpha");
    assert!(
        !body["description_entity_id"]
            .as_str()
            .unwrap_or("")
            .is_empty()
    );
    assert!(!body["created_at"].as_str().unwrap_or("").is_empty());
}

#[tokio::test]
async fn get_project_returns_200_with_same_body() {
    let addr = spawn_app().await;
    let client = reqwest::Client::new();

    let post_resp = client
        .post(format!("http://{addr}/projects"))
        .json(&json!({ "name": "Beta", "description": "second project" }))
        .send()
        .await
        .unwrap();
    assert_eq!(post_resp.status(), 201);
    let created: serde_json::Value = post_resp.json().await.unwrap();
    let id = created["id"].as_str().unwrap();

    let get_resp = client
        .get(format!("http://{addr}/projects/{id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(get_resp.status(), 200);
    let fetched: serde_json::Value = get_resp.json().await.unwrap();
    assert_eq!(fetched, created);
}

#[tokio::test]
async fn get_project_unknown_id_returns_404() {
    let addr = spawn_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!(
            "http://{addr}/projects/00000000-0000-0000-0000-000000000000"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn create_project_empty_name_returns_400() {
    let addr = spawn_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("http://{addr}/projects"))
        .json(&json!({ "name": "", "description": "some description" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body["error"].as_str().unwrap_or("").contains("name"),
        "error should mention 'name'"
    );
}

#[tokio::test]
async fn create_project_empty_description_returns_400() {
    let addr = spawn_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("http://{addr}/projects"))
        .json(&json!({ "name": "Gamma", "description": "" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body["error"].as_str().unwrap_or("").contains("description"),
        "error should mention 'description'"
    );
}

#[tokio::test]
async fn get_entity_returns_200_with_description_entity() {
    let addr = spawn_app().await;
    let client = reqwest::Client::new();

    let post_resp = client
        .post(format!("http://{addr}/projects"))
        .json(&json!({ "name": "Delta", "description": "entity fetch test" }))
        .send()
        .await
        .unwrap();
    assert_eq!(post_resp.status(), 201);
    let project: serde_json::Value = post_resp.json().await.unwrap();
    let entity_id = project["description_entity_id"].as_str().unwrap();

    let resp = client
        .get(format!("http://{addr}/entities/{entity_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let entity: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(entity["id"], entity_id);
    assert_eq!(entity["entity_type"], "raw");
    assert!(!entity["content"].is_null());
}

#[tokio::test]
async fn list_projects_returns_200_with_array() {
    let addr = spawn_app().await;
    let client = reqwest::Client::new();

    client
        .post(format!("http://{addr}/projects"))
        .json(&json!({ "name": "Eta", "description": "first" }))
        .send()
        .await
        .unwrap();
    client
        .post(format!("http://{addr}/projects"))
        .json(&json!({ "name": "Theta", "description": "second" }))
        .send()
        .await
        .unwrap();

    let resp = client
        .get(format!("http://{addr}/projects"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.is_array());
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn list_project_entities_returns_200_with_array() {
    let addr = spawn_app().await;
    let client = reqwest::Client::new();

    let post_resp = client
        .post(format!("http://{addr}/projects"))
        .json(&json!({ "name": "Iota", "description": "entities test" }))
        .send()
        .await
        .unwrap();
    let project: serde_json::Value = post_resp.json().await.unwrap();
    let id = project["id"].as_str().unwrap();

    let resp = client
        .get(format!("http://{addr}/projects/{id}/entities"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.is_array());
    assert_eq!(
        body.as_array().unwrap().len(),
        1,
        "should have one description entity"
    );
}

#[tokio::test]
async fn list_project_entities_unknown_project_returns_404() {
    let addr = spawn_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!(
            "http://{addr}/projects/00000000-0000-0000-0000-000000000000/entities"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn get_entity_unknown_id_returns_404() {
    let addr = spawn_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!(
            "http://{addr}/entities/00000000-0000-0000-0000-000000000000"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}
