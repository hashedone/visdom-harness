use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use tracing::instrument;
use uuid::Uuid;

use crate::AppState;
use crate::entities::Page;
use crate::error::AppError;
use crate::llm::LlmClient;
use crate::questions::{self, Answer, Question};

#[derive(Debug, Deserialize)]
pub struct ListQuestionsParams {
    pub status: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, Deserialize)]
pub struct AnswerRequest {
    pub answer: String,
}

/// GET /projects/:project_id/questions[?status=open|answered&limit=N&offset=N]
#[instrument(skip(state), fields(project_id = %project_id))]
pub async fn list_questions<L: LlmClient>(
    State(state): State<AppState<L>>,
    Path(project_id): Path<Uuid>,
    Query(params): Query<ListQuestionsParams>,
) -> Result<Json<Page<Question>>, AppError> {
    // Validate project exists.
    crate::projects::get(&state.pool, project_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let status_filter = params.status.as_deref();
    let page = questions::list(
        &state.pool,
        project_id,
        status_filter,
        params.limit,
        params.offset,
    )
    .await?;
    Ok(Json(page))
}

/// GET /questions/:id
#[instrument(skip(state), fields(id = %id))]
pub async fn get_question<L: LlmClient>(
    State(state): State<AppState<L>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Question>, AppError> {
    let q = questions::get(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(q) as Json<Question>)
}

/// POST /questions/:id/answer
/// Body: { "answer": "..." }
#[instrument(skip(state), fields(id = %id))]
pub async fn post_answer<L: LlmClient>(
    State(state): State<AppState<L>>,
    Path(id): Path<Uuid>,
    Json(body): Json<AnswerRequest>,
) -> Result<(StatusCode, Json<Answer>), AppError> {
    if body.answer.trim().is_empty() {
        return Err(AppError::EmptyDescription); // reuse — "answer text is required"
    }

    let question: Question = questions::get(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;

    if question.status == "answered" {
        // Additional answers are allowed (multiple integrations may reply).
        // We still record it but return 200 rather than 201.
        let ans = questions::answer(&state.pool, &question, &body.answer).await?;
        return Ok((StatusCode::OK, Json(ans)));
    }

    let ans = questions::answer(&state.pool, &question, &body.answer).await?;
    Ok((StatusCode::CREATED, Json(ans)))
}
