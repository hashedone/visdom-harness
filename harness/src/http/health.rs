use axum::Json;
use serde_json::{Value, json};
use tracing::instrument;

#[instrument(name = "http.health")]
pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
