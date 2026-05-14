use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use tracing::info;
use uuid::Uuid;

use crate::{
    error::AppError,
    llm::{InferenceMessage, InferenceResult},
};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InferenceRecord {
    pub id: String,
    pub system_prompt: String,
    pub request_messages_json: String,
    pub response_text: String,
    pub tool_calls_json: String,
    pub created_at: String,
}

pub async fn record(
    pool: &SqlitePool,
    result: InferenceResult,
    system_prompt: &str,
    messages: &[InferenceMessage],
) -> Result<InferenceRecord, AppError> {
    let id = Uuid::new_v4().to_string();
    let request_messages_json = serde_json::to_string(messages)
        .map_err(|e| AppError::Internal(eyre::Report::from(e)))?;
    let tool_calls_json = serde_json::to_string(&result.tool_calls)
        .map_err(|e| AppError::Internal(eyre::Report::from(e)))?;

    let row = sqlx::query_as::<_, InferenceRecord>(
        r#"
        INSERT INTO inferences (id, system_prompt, request_messages_json, response_text, tool_calls_json)
        VALUES (?, ?, ?, ?, ?)
        RETURNING id, system_prompt, request_messages_json, response_text, tool_calls_json, created_at
        "#,
    )
    .bind(&id)
    .bind(system_prompt)
    .bind(&request_messages_json)
    .bind(&result.response_text)
    .bind(&tool_calls_json)
    .fetch_one(pool)
    .await?;

    info!(
        inference_id = %row.id,
        system_prompt_len = system_prompt.len(),
        tool_call_count = result.tool_calls.len(),
        "inference recorded"
    );

    Ok(row)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<InferenceRecord>, AppError> {
    let row = sqlx::query_as::<_, InferenceRecord>(
        "SELECT id, system_prompt, request_messages_json, response_text, tool_calls_json, created_at FROM inferences WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn list(pool: &SqlitePool, limit: i64) -> Result<Vec<InferenceRecord>, AppError> {
    let rows = sqlx::query_as::<_, InferenceRecord>(
        "SELECT id, system_prompt, request_messages_json, response_text, tool_calls_json, created_at FROM inferences ORDER BY created_at DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
