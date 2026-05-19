use std::net::SocketAddr;

use serde_json::{Value, json};
use tokio::net::TcpListener;
use visdom_harness::error::AppError;
use visdom_harness::llm::{InferenceMessage, InferenceResult, LlmClient, ToolSpec};
use visdom_harness::{AppState, db};

#[derive(Clone)]
struct NoopLlm;

impl LlmClient for NoopLlm {
    async fn infer(
        &self,
        _system_prompt: &str,
        _messages: &[InferenceMessage],
        _tools: &[ToolSpec],
    ) -> Result<InferenceResult, AppError> {
        Err(AppError::MissingApiKey)
    }
}

async fn spawn_app() -> SocketAddr {
    let pool = db::in_memory_pool().await.unwrap();
    let state = AppState {
        pool,
        llm: NoopLlm,
        integrations: visdom_harness::IntegrationRegistry::new(),
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = visdom_harness::build_app(state);
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

async fn create_project(client: &reqwest::Client, addr: SocketAddr) -> Value {
    client
        .post(format!("http://{addr}/projects"))
        .json(&json!({"name": "Test", "description": "A test project"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_questions_empty_for_new_project() {
    let _addr = spawn_app().await;
    let client = reqwest::Client::new();
    let project = create_project(&client, addr).await;
    let project_id = project["id"].as_str().unwrap();

    let resp = client
        .get(format!("http://{addr}/projects/{project_id}/questions"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["total"], 0);
    assert!(body["items"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn list_questions_unknown_project_returns_404() {
    let _addr = spawn_app().await;
    let client = reqwest::Client::new();
    let fake_id = uuid::Uuid::new_v4();

    let resp = client
        .get(format!("http://{addr}/projects/{fake_id}/questions"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn get_question_unknown_returns_404() {
    let _addr = spawn_app().await;
    let client = reqwest::Client::new();
    let fake_id = uuid::Uuid::new_v4();

    let resp = client
        .get(format!("http://{addr}/questions/{fake_id}"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn answer_unknown_question_returns_404() {
    let _addr = spawn_app().await;
    let client = reqwest::Client::new();
    let fake_id = uuid::Uuid::new_v4();

    let resp = client
        .post(format!("http://{addr}/questions/{fake_id}/answer"))
        .json(&json!({"answer": "some answer"}))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn answer_empty_text_returns_400() {
    let _addr = spawn_app().await;
    let client = reqwest::Client::new();

    // Create a question via the repo directly would require exposing it,
    // so we test the validation path by trying a known-bad payload
    // against a question that doesn't exist yet — 404 beats 400 in routing,
    // so we use the empty-answer check on an existing question.
    // First, create a project, then seed a question via the repo helper.
    let pool = db::in_memory_pool().await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    let proj = visdom_harness::projects::create_in_tx(&mut tx, "Proj", "desc")
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let q = visdom_harness::questions::create(
        &pool,
        proj.id,
        "What is the scope?",
        "Use the answer to update context.",
    )
    .await
    .unwrap();

    // Spin up app with this pool.
    let state = AppState {
        pool,
        llm: NoopLlm,
        integrations: visdom_harness::IntegrationRegistry::new(),
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr2 = listener.local_addr().unwrap();
    let app = visdom_harness::build_app(state);
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let resp = client
        .post(format!("http://{addr2}/questions/{}/answer", q.id))
        .json(&json!({"answer": "   "}))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn answer_question_creates_answer_and_marks_answered() {
    let _addr = spawn_app().await;
    let client = reqwest::Client::new();
    let project = create_project(&client, addr).await;
    let project_id = project["id"].as_str().unwrap();
    let project_uuid: uuid::Uuid = project_id.parse().unwrap();

    // Seed a question directly via repo.
    let pool = db::in_memory_pool().await.unwrap();
    // We need the same pool as the server — use the HTTP API path instead.
    // Since there's no POST /questions endpoint (questions are created by the loop),
    // we call answer via a fresh app that shares pool state.
    let pool2 = db::in_memory_pool().await.unwrap();
    let mut tx = pool2.begin().await.unwrap();
    let proj = visdom_harness::projects::create_in_tx(&mut tx, "P", "d")
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let q = visdom_harness::questions::create(
        &pool2,
        proj.id,
        "How big?",
        "Update context with scale info.",
    )
    .await
    .unwrap();

    let state = AppState {
        pool: pool2,
        llm: NoopLlm,
        integrations: visdom_harness::IntegrationRegistry::new(),
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr2 = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, visdom_harness::build_app(state))
            .await
            .unwrap()
    });

    // First answer → 201
    let resp = client
        .post(format!("http://{addr2}/questions/{}/answer", q.id))
        .json(&json!({"answer": "About 100 users"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let ans: Value = resp.json().await.unwrap();
    assert_eq!(ans["question_id"], q.id.to_string());

    // Second answer to same question → 200 (additional answer allowed)
    let resp2 = client
        .post(format!("http://{addr2}/questions/{}/answer", q.id))
        .json(&json!({"answer": "Actually more like 500"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp2.status(), 200);

    // Verify question now shows as answered
    let q_resp = client
        .get(format!("http://{addr2}/questions/{}", q.id))
        .send()
        .await
        .unwrap();
    assert_eq!(q_resp.status(), 200);
    let q_body: Value = q_resp.json().await.unwrap();
    assert_eq!(q_body["status"], "answered");

    // Suppress unused variable warning
    let _ = project_uuid;
    let _ = pool;
}

#[tokio::test]
async fn list_questions_status_filter() {
    let pool = db::in_memory_pool().await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    let proj = visdom_harness::projects::create_in_tx(&mut tx, "P", "d")
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let q1 = visdom_harness::questions::create(&pool, proj.id, "Q1?", "inst1")
        .await
        .unwrap();
    let _q2 = visdom_harness::questions::create(&pool, proj.id, "Q2?", "inst2")
        .await
        .unwrap();

    // Answer q1
    visdom_harness::questions::answer(&pool, &q1, "Answer to Q1")
        .await
        .unwrap();

    let state = AppState {
        pool,
        llm: NoopLlm,
        integrations: visdom_harness::IntegrationRegistry::new(),
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, visdom_harness::build_app(state))
            .await
            .unwrap()
    });
    let client = reqwest::Client::new();

    // open filter → 1 result
    let resp = client
        .get(format!(
            "http://{addr}/projects/{}/questions?status=open",
            proj.id
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["total"], 1);

    // answered filter → 1 result
    let resp = client
        .get(format!(
            "http://{addr}/projects/{}/questions?status=answered",
            proj.id
        ))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["total"], 1);

    // no filter → 2 results
    let resp = client
        .get(format!("http://{addr}/projects/{}/questions", proj.id))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["total"], 2);
}
