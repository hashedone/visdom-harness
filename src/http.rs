pub mod debug;
pub mod health;

use axum::Router;
use axum::routing::{get, post};

use crate::AppState;
use crate::llm::LlmClient;

pub fn router<L: LlmClient>(state: AppState<L>) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/debug/infer", post(debug::post_infer::<L>))
        .route("/debug/inferences", get(debug::list_inferences::<L>))
        .route("/debug/inferences/:id", get(debug::get_inference::<L>))
        .with_state(state)
}
