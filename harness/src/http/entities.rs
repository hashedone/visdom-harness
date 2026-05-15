use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use tracing::instrument;
use uuid::Uuid;

use crate::AppState;
use crate::entities::{self, Entity, Page};
use crate::error::AppError;
use crate::llm::LlmClient;

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

pub async fn list_entities<L: LlmClient>(
    State(state): State<AppState<L>>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Page<Entity>>, AppError> {
    let page = entities::list(&state.pool, pagination.limit, pagination.offset).await?;
    Ok(Json(page))
}

#[instrument(skip(state), fields(entity_id = %id))]
pub async fn get_entity<L: LlmClient>(
    State(state): State<AppState<L>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Entity>, AppError> {
    let entity = entities::get(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(entity))
}
