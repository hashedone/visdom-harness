pub mod db;
pub mod entities;
pub mod error;
pub mod http;
pub mod inferences;
pub mod llm;
pub mod projects;
pub mod telemetry;

use axum::Router;
use sqlx::SqlitePool;
use tower_http::cors::CorsLayer;

use crate::llm::LlmClient;

#[derive(Clone)]
pub struct AppState<L: LlmClient> {
    pub pool: SqlitePool,
    pub llm: L,
}

pub fn build_app<L: LlmClient>(state: AppState<L>) -> Router {
    http::router(state).layer(CorsLayer::permissive())
}
