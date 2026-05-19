pub mod db;
pub mod entities;
pub mod error;
pub mod http;
pub mod inferences;
pub mod integrations;
pub mod llm;
pub mod projects;
pub mod telemetry;

use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::ws::Message;
use sqlx::SqlitePool;
use tokio::sync::mpsc::UnboundedSender;
use tower_http::cors::CorsLayer;

use crate::llm::LlmClient;

/// Registry of active integration WebSocket connections.
/// Senders are removed automatically when their connection drops.
#[derive(Clone, Default)]
pub struct IntegrationRegistry {
    pub senders: Arc<Mutex<Vec<UnboundedSender<Message>>>>,
}

impl IntegrationRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fan out a message to all connected integrations.
    /// Dead senders (disconnected clients) are pruned in-place.
    pub fn fan_out(&self, msg: Message) {
        let mut guard = self.senders.lock().expect("registry lock poisoned");
        guard.retain(|tx| tx.send(msg.clone()).is_ok());
    }
}

#[derive(Clone)]
pub struct AppState<L: LlmClient> {
    pub pool: SqlitePool,
    pub llm: L,
    pub integrations: IntegrationRegistry,
}

pub fn build_app<L: LlmClient>(state: AppState<L>) -> Router {
    http::router(state).layer(CorsLayer::permissive())
}
