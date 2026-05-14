use axum::Json;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::instrument;

use crate::AppState;
use crate::error::AppError;
use crate::inferences::{self, InferenceRecord};
use crate::llm::{InferenceMessage, MessageRole, ToolSpec};

#[derive(Debug, Deserialize)]
pub struct InferRequest {
    pub prompt: String,
}

#[derive(Debug, Serialize)]
pub struct InferResponse {
    pub id: String,
}

#[instrument(skip(state), fields(prompt_chars))]
pub async fn post_infer(
    State(state): State<AppState>,
    Json(body): Json<InferRequest>,
) -> Result<Json<InferResponse>, AppError> {
    if body.prompt.is_empty() {
        return Err(AppError::EmptyPrompt);
    }

    tracing::Span::current().record("prompt_chars", body.prompt.len());

    let system_prompt = "You are a helpful assistant. Use tools when relevant.";
    let messages = vec![InferenceMessage {
        role: MessageRole::User,
        content: body.prompt.clone(),
    }];
    let weather_tool = ToolSpec {
        name: "get_weather".to_string(),
        description: "Look up the current weather in a city".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "city": { "type": "string" }
            },
            "required": ["city"]
        }),
    };

    let result = state
        .llm
        .infer(system_prompt, &messages, &[weather_tool])
        .await?;

    let record = inferences::record(&state.pool, result, system_prompt, &messages).await?;

    Ok(Json(InferResponse { id: record.id }))
}

pub async fn get_inference(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<InferenceRecord>, AppError> {
    let record = inferences::get(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(record))
}

pub async fn list_inferences(
    State(state): State<AppState>,
) -> Result<Json<Vec<InferenceRecord>>, AppError> {
    let records = inferences::list(&state.pool, 100).await?;
    Ok(Json(records))
}
