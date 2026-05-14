pub mod debug;
pub mod health;

use axum::Router;
use axum::routing::{get, post};

use crate::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/debug/infer", post(debug::post_infer))
        .route("/debug/inferences", get(debug::list_inferences))
        .route("/debug/inferences/:id", get(debug::get_inference))
        .with_state(state)
}
