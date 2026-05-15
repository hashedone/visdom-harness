use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use rig::completion::CompletionError;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,

    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("internal error: {0}")]
    Internal(#[from] eyre::Report),

    #[error("llm error: {0}")]
    Llm(#[from] CompletionError),

    #[error("llm api key not configured")]
    MissingApiKey,

    #[error("prompt must not be empty")]
    EmptyPrompt,

    #[error("name must not be empty")]
    EmptyName,

    #[error("description must not be empty")]
    EmptyDescription,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Llm(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            AppError::MissingApiKey => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::EmptyPrompt | AppError::EmptyName | AppError::EmptyDescription => {
                (StatusCode::BAD_REQUEST, self.to_string())
            }
            AppError::Db(_) | AppError::Migration(_) | AppError::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
