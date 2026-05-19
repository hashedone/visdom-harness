pub mod debug;
pub mod entities;
pub mod health;
pub mod projects;

use axum::Router;
use axum::routing::{get, post};

use crate::AppState;
use crate::integrations;
use crate::llm::LlmClient;

pub fn router<L: LlmClient>(state: AppState<L>) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/debug/infer", post(debug::post_infer::<L>))
        .route("/debug/inferences", get(debug::list_inferences::<L>))
        .route("/debug/inferences/:id", get(debug::get_inference::<L>))
        .route(
            "/projects",
            get(projects::list_projects::<L>).post(projects::create_project::<L>),
        )
        .route("/projects/:id", get(projects::get_project::<L>))
        .route(
            "/projects/:id/entities",
            get(projects::list_project_entities::<L>),
        )
        .route("/entities", get(entities::list_entities::<L>))
        .route("/entities/:id", get(entities::get_entity::<L>))
        .route("/integrations/connect", get(integrations::connect::<L>))
        .with_state(state)
}
