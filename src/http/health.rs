use axum::Json;
use serde_json::{json, Value};
use tracing::instrument;

#[instrument]
pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
