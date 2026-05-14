pub mod db;
pub mod error;
pub mod http;
pub mod inferences;
pub mod llm;
pub mod telemetry;

use std::sync::Arc;

use axum::Router;
use sqlx::SqlitePool;

use crate::llm::LlmClient;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub llm: Arc<dyn LlmClient + Send + Sync>,
}

pub fn build_app(state: AppState) -> Router {
    http::router(state)
}
