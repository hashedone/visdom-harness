use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::Deserialize;
use tracing::instrument;

use crate::AppState;
use crate::error::AppError;
use crate::llm::LlmClient;
use crate::projects::{self, Project};

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: String,
}

#[instrument(skip(state), fields(name_chars, description_chars))]
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

    tracing::Span::current().record("name_chars", body.name.len());
    tracing::Span::current().record("description_chars", body.description.len());

    let project = projects::create(&state.pool, &body.name, &body.description).await?;
    tracing::info!(project_id = %project.id, "project created");
    Ok((StatusCode::CREATED, Json(project)))
}

#[instrument(skip(state), fields(project_id = %id))]
pub async fn get_project<L: LlmClient>(
    State(state): State<AppState<L>>,
    Path(id): Path<String>,
) -> Result<Json<Project>, AppError> {
    let project = projects::get(&state.pool, &id).await?.ok_or(AppError::NotFound)?;
    Ok(Json(project))
}
