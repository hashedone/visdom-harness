use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use tracing::instrument;
use uuid::Uuid;

use crate::AppState;
use crate::entities::{self, Entity, Page};
use crate::error::AppError;
use crate::http::entities::PaginationParams;
use crate::llm::LlmClient;
use crate::projects::{self, Project};

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: String,
}

#[instrument(skip(state), fields(name, description_chars))]
pub async fn create_project<L: LlmClient>(
    State(state): State<AppState<L>>,
    Json(body): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<Project>), AppError> {
    if body.name.trim().is_empty() {
        return Err(AppError::EmptyName);
    }
    if body.description.trim().is_empty() {
        return Err(AppError::EmptyDescription);
    }

    tracing::Span::current().record("name", body.name.as_str());
    tracing::Span::current().record("description_chars", body.description.len());

    let mut tx = state.pool.begin().await?;
    let project = projects::create_in_tx(&mut tx, &body.name, &body.description).await?;
    tx.commit().await?;

    tracing::info!(
        project_id = %project.id,
        description_entity_id = %project.description_entity_id,
        "project created"
    );
    Ok((StatusCode::CREATED, Json(project)))
}

#[instrument(skip(state))]
pub async fn list_projects<L: LlmClient>(
    State(state): State<AppState<L>>,
) -> Result<Json<Vec<Project>>, AppError> {
    let projects = projects::list(&state.pool).await?;
    Ok(Json(projects))
}

#[instrument(skip(state), fields(project_id = %id))]
pub async fn get_project<L: LlmClient>(
    State(state): State<AppState<L>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Project>, AppError> {
    let project = projects::get(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(project))
}

#[instrument(skip(state), fields(project_id = %id))]
pub async fn list_project_entities<L: LlmClient>(
    State(state): State<AppState<L>>,
    Path(id): Path<Uuid>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Page<Entity>>, AppError> {
    if !projects::exists(&state.pool, id).await? {
        return Err(AppError::NotFound);
    }
    let page =
        entities::list_by_project(&state.pool, id, pagination.limit, pagination.offset).await?;
    Ok(Json(page))
}
