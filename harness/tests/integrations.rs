use std::time::Duration;

use futures_util::StreamExt;
use tokio::net::TcpListener;
use tokio::time::timeout;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use visdom_harness::error::AppError;
use visdom_harness::llm::{InferenceMessage, InferenceResult, LlmClient, ToolSpec};
use visdom_harness::{AppState, IntegrationRegistry, db};

// ---------------------------------------------------------------------------
// Minimal no-op LLM stub (same pattern as other integration tests)
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct NoopLlmClient;

impl LlmClient for NoopLlmClient {
    async fn infer(
        &self,
        _system_prompt: &str,
        _messages: &[InferenceMessage],
        _tools: &[ToolSpec],
    ) -> Result<InferenceResult, AppError> {
        Err(AppError::MissingApiKey)
    }
}

async fn spawn_app() -> String {
    let pool = db::in_memory_pool().await.unwrap();
    let state = AppState {
        pool,
        llm: NoopLlmClient,
        integrations: IntegrationRegistry::new(),
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = visdom_harness::build_app(state);
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("ws://127.0.0.1:{}", addr.port())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Connecting to /integrations/connect upgrades to WebSocket successfully.
#[tokio::test]
async fn connect_upgrades_to_websocket() {
    let base = spawn_app().await;
    let url = format!("{base}/integrations/connect");

    let result = connect_async(&url).await;
    assert!(
        result.is_ok(),
        "WebSocket upgrade failed: {:?}",
        result.err()
    );
}

/// The connection stays open after upgrade — no immediate close frame.
#[tokio::test]
async fn connection_stays_open_after_upgrade() {
    let base = spawn_app().await;
    let url = format!("{base}/integrations/connect");

    let (mut ws, _) = connect_async(&url).await.expect("WebSocket upgrade failed");

    // The connection must not close within 200ms.
    let result = timeout(Duration::from_millis(200), ws.next()).await;
    match result {
        Err(_elapsed) => {
            // Timeout = no close frame = connection is alive. ✓
        }
        Ok(Some(Ok(Message::Ping(_)))) => {
            // A ping arrived (valid — unlikely at 30s interval but fine). ✓
        }
        Ok(Some(Ok(msg))) => {
            panic!("unexpected message immediately after connect: {msg:?}");
        }
        Ok(Some(Err(e))) => panic!("WebSocket error: {e}"),
        Ok(None) => panic!("connection closed immediately after upgrade"),
    }
}

/// Multiple integrations can connect simultaneously.
#[tokio::test]
async fn multiple_integrations_can_connect() {
    let base = spawn_app().await;
    let url = format!("{base}/integrations/connect");

    let (ws1, _) = connect_async(&url).await.expect("first connect failed");
    let (ws2, _) = connect_async(&url).await.expect("second connect failed");
    let (ws3, _) = connect_async(&url).await.expect("third connect failed");

    drop(ws1);
    drop(ws2);
    drop(ws3);
}
