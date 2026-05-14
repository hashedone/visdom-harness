use rig::completion::{AssistantContent, CompletionModel, ToolDefinition};
use rig::message::{Message, UserContent};
use rig::one_or_many::OneOrMany;
use rig::providers::anthropic::{ClientBuilder, completion::CLAUDE_3_7_SONNET};

use crate::error::AppError;

use super::{InferenceMessage, InferenceResult, LlmClient, MessageRole, ToolCallRecord, ToolSpec};

#[derive(Clone)]
pub struct AnthropicLlmClient {
    model: rig::providers::anthropic::completion::CompletionModel,
}

impl std::fmt::Debug for AnthropicLlmClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnthropicLlmClient")
            .field("model", &self.model.model)
            .finish()
    }
}

impl AnthropicLlmClient {
    pub fn from_env() -> Result<Self, AppError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| AppError::MissingApiKey)?;
        let client = ClientBuilder::new(&api_key).build();
        Ok(Self {
            model: client.completion_model(CLAUDE_3_7_SONNET),
        })
    }
}

impl LlmClient for AnthropicLlmClient {
    async fn infer(
        &self,
        system_prompt: &str,
        messages: &[InferenceMessage],
        tools: &[ToolSpec],
    ) -> Result<InferenceResult, AppError> {
        let prompt_text = messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default();

        let history: Vec<Message> = messages[..messages.len().saturating_sub(1)]
            .iter()
            .map(|m| match m.role {
                MessageRole::User => Message::User {
                    content: OneOrMany::one(UserContent::text(&m.content)),
                },
                MessageRole::Assistant => Message::Assistant {
                    content: OneOrMany::one(AssistantContent::text(&m.content)),
                },
            })
            .collect();

        let tool_defs: Vec<ToolDefinition> = tools
            .iter()
            .map(|t| ToolDefinition {
                name: t.name.clone(),
                description: t.description.clone(),
                parameters: t.parameters.clone(),
            })
            .collect();

        let prompt_msg = Message::User {
            content: OneOrMany::one(UserContent::text(&prompt_text)),
        };

        let request = self
            .model
            .completion_request(prompt_msg)
            .preamble(system_prompt.to_string())
            .messages(history)
            .tools(tool_defs)
            .build();

        let response = self.model.completion(request).await?;

        let (tool_calls, texts): (Vec<ToolCallRecord>, Vec<String>) =
            response.choice.into_iter().fold(
                (Vec::new(), Vec::new()),
                |(mut tool_calls, mut texts), item| {
                    match item {
                        AssistantContent::ToolCall(tc) => tool_calls.push(ToolCallRecord {
                            id: tc.id,
                            name: tc.function.name,
                            arguments: tc.function.arguments,
                        }),
                        AssistantContent::Text(t) => texts.push(t.text),
                    }
                    (tool_calls, texts)
                },
            );

        Ok(InferenceResult {
            prompt_text,
            response_text: texts.into_iter().next().unwrap_or_default(),
            tool_calls,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "current_thread")]
    async fn from_env_returns_llm_error_when_key_absent() {
        // Skip if user already has the key set — we'd have to unset it and that is
        // not safe in a multi-threaded harness.
        if std::env::var("ANTHROPIC_API_KEY").is_ok() {
            return;
        }
        // Belt-and-suspenders: ensure key is absent.
        // SAFETY: single-threaded test runtime (current_thread flavor).
        unsafe {
            std::env::remove_var("ANTHROPIC_API_KEY");
        }
        let result = AnthropicLlmClient::from_env();
        assert!(
            matches!(result, Err(AppError::MissingApiKey)),
            "expected MissingApiKey error, got: {:?}",
            result
        );
    }
}
