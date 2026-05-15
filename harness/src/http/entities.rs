use axum::Json;
use axum::extract::{Path, State};
use tracing::instrument;
use uuid::Uuid;

use crate::AppState;
use crate::entities::{self, Entity};
use crate::error::AppError;
use crate::llm::LlmClient;

pub async fn list_entities<L: LlmClient>(
    State(state): State<AppState<L>>,
) -> Result<Json<Vec<Entity>>, AppError> {
    let entities = entities::list(&state.pool, 500).await?;
    Ok(Json(entities))
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
