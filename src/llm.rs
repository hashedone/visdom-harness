pub mod anthropic;

use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub prompt_text: String,
    pub response_text: String,
    pub tool_calls: Vec<ToolCallRecord>,
}

#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    async fn infer(
        &self,
        system_prompt: &str,
        messages: &[InferenceMessage],
        tools: &[ToolSpec],
    ) -> Result<InferenceResult, AppError>;
}
