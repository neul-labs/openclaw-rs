//! OpenAI API provider.

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use crate::traits::{
    ChunkType, CompletionRequest, CompletionResponse, ContentBlock, MessageContent, Provider,
    ProviderError, Role, StopReason, StreamingChunk,
};
use openclaw_core::secrets::ApiKey;
use openclaw_core::types::TokenUsage;

const DEFAULT_BASE_URL: &str = "https://api.openai.com";

/// OpenAI API provider.
pub struct OpenAIProvider {
    client: Client,
    api_key: ApiKey,
    base_url: String,
    org_id: Option<String>,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider.
    #[must_use]
    pub fn new(api_key: ApiKey) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: DEFAULT_BASE_URL.to_string(),
            org_id: None,
        }
    }

    /// Create with custom base URL (for Azure or compatible APIs).
    #[must_use]
    pub fn with_base_url(api_key: ApiKey, base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: base_url.into(),
            org_id: None,
        }
    }

    /// Set organization ID.
    #[must_use]
    pub fn with_org_id(mut self, org_id: impl Into<String>) -> Self {
        self.org_id = Some(org_id.into());
        self
    }

    /// Convert our request format to OpenAI's API format.
    fn to_openai_request(&self, request: &CompletionRequest) -> OpenAIRequest {
        let mut messages: Vec<OpenAIMessage> = Vec::new();

        // Add system message if present
        if let Some(system) = &request.system {
            messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: Some(OpenAIContent::Text(system.clone())),
                tool_calls: None,
                tool_call_id: None,
            });
        }

        // Convert messages
        for msg in &request.messages {
            let openai_msg = match msg.role {
                Role::System => OpenAIMessage {
                    role: "system".to_string(),
                    content: Some(content_to_openai(&msg.content)),
                    tool_calls: None,
                    tool_call_id: None,
                },
                Role::User => OpenAIMessage {
                    role: "user".to_string(),
                    content: Some(content_to_openai(&msg.content)),
                    tool_calls: None,
                    tool_call_id: None,
                },
                Role::Assistant => {
                    let (content, tool_calls) = extract_tool_calls(&msg.content);
                    OpenAIMessage {
                        role: "assistant".to_string(),
                        content,
                        tool_calls,
                        tool_call_id: None,
                    }
                }
                Role::Tool => {
                    if let MessageContent::Blocks(blocks) = &msg.content {
                        for block in blocks {
                            if let ContentBlock::ToolResult {
                                tool_use_id,
                                content,
                                ..
                            } = block
                            {
                                messages.push(OpenAIMessage {
                                    role: "tool".to_string(),
                                    content: Some(OpenAIContent::Text(content.clone())),
                                    tool_calls: None,
                                    tool_call_id: Some(tool_use_id.clone()),
                                });
                            }
                        }
                    }
                    continue;
                }
            };
            messages.push(openai_msg);
        }

        let tools = request.tools.as_ref().map(|tools| {
            tools
                .iter()
                .map(|t| OpenAITool {
                    tool_type: "function".to_string(),
                    function: OpenAIFunction {
                        name: t.name.clone(),
                        description: t.description.clone(),
                        parameters: t.input_schema.clone(),
                    },
                })
                .collect()
        });

        OpenAIRequest {
            model: request.model.clone(),
            messages,
            max_tokens: Some(request.max_tokens),
            temperature: Some(request.temperature),
            stop: request.stop.clone(),
            tools,
            stream: Some(false),
        }
    }
}

fn content_to_openai(content: &MessageContent) -> OpenAIContent {
    match content {
        MessageContent::Text(text) => OpenAIContent::Text(text.clone()),
        MessageContent::Blocks(blocks) => {
            let parts: Vec<OpenAIContentPart> = blocks
                .iter()
                .filter_map(|b| match b {
                    ContentBlock::Text { text } => {
                        Some(OpenAIContentPart::Text { text: text.clone() })
                    }
                    ContentBlock::Image { source } => Some(OpenAIContentPart::ImageUrl {
                        image_url: OpenAIImageUrl {
                            url: format!("data:{};base64,{}", source.media_type, source.data),
                        },
                    }),
                    _ => None,
                })
                .collect();
            OpenAIContent::Parts(parts)
        }
    }
}

fn extract_tool_calls(
    content: &MessageContent,
) -> (Option<OpenAIContent>, Option<Vec<OpenAIToolCall>>) {
    match content {
        MessageContent::Text(text) => (Some(OpenAIContent::Text(text.clone())), None),
        MessageContent::Blocks(blocks) => {
            let mut text_parts = Vec::new();
            let mut tool_calls = Vec::new();

            for block in blocks {
                match block {
                    ContentBlock::Text { text } => text_parts.push(text.clone()),
                    ContentBlock::ToolUse { id, name, input } => {
                        tool_calls.push(OpenAIToolCall {
                            id: id.clone(),
                            call_type: "function".to_string(),
                            function: OpenAIFunctionCall {
                                name: name.clone(),
                                arguments: serde_json::to_string(input).unwrap_or_default(),
                            },
                        });
                    }
                    _ => {}
                }
            }

            let content = if text_parts.is_empty() {
                None
            } else {
                Some(OpenAIContent::Text(text_parts.join("\n")))
            };

            let tool_calls = if tool_calls.is_empty() {
                None
            } else {
                Some(tool_calls)
            };

            (content, tool_calls)
        }
    }
}

#[async_trait]
impl Provider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    async fn list_models(&self) -> Result<Vec<String>, ProviderError> {
        let url = format!("{}/v1/models", self.base_url);

        let mut req = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key.expose()));

        if let Some(org) = &self.org_id {
            req = req.header("OpenAI-Organization", org);
        }

        let response = req.send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(ProviderError::Api { status, message });
        }

        let result: OpenAIModelsResponse = response.json().await?;
        Ok(result.data.into_iter().map(|m| m.id).collect())
    }

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError> {
        let url = format!("{}/v1/chat/completions", self.base_url);
        let openai_request = self.to_openai_request(&request);

        let mut req = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key.expose()))
            .header("Content-Type", "application/json");

        if let Some(org) = &self.org_id {
            req = req.header("OpenAI-Organization", org);
        }

        let response = req.json(&openai_request).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();

            if status == 429 {
                let retry_after = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(60);
                return Err(ProviderError::RateLimited {
                    retry_after_secs: retry_after,
                });
            }

            let message = response.text().await.unwrap_or_default();
            return Err(ProviderError::Api { status, message });
        }

        let result: OpenAIResponse = response.json().await?;
        Ok(result.into())
    }

    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<
        Pin<Box<dyn Stream<Item = Result<StreamingChunk, ProviderError>> + Send>>,
        ProviderError,
    > {
        let url = format!("{}/v1/chat/completions", self.base_url);
        let mut openai_request = self.to_openai_request(&request);
        openai_request.stream = Some(true);

        let mut req = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key.expose()))
            .header("Content-Type", "application/json");

        if let Some(org) = &self.org_id {
            req = req.header("OpenAI-Organization", org);
        }

        let response = req.json(&openai_request).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(ProviderError::Api { status, message });
        }

        let stream = response.bytes_stream().map(move |result| match result {
            Ok(bytes) => {
                let text = String::from_utf8_lossy(&bytes);
                parse_sse_event(&text)
            }
            Err(e) => Err(ProviderError::Network(e)),
        });

        Ok(Box::pin(stream))
    }
}

fn parse_sse_event(text: &str) -> Result<StreamingChunk, ProviderError> {
    for line in text.lines() {
        if let Some(data) = line.strip_prefix("data: ") {
            if data == "[DONE]" {
                return Ok(StreamingChunk {
                    chunk_type: ChunkType::MessageStop,
                    delta: None,
                    index: None,
                });
            }

            if let Ok(event) = serde_json::from_str::<OpenAIStreamEvent>(data) {
                if let Some(choice) = event.choices.first() {
                    return Ok(StreamingChunk {
                        chunk_type: if choice.finish_reason.is_some() {
                            ChunkType::MessageStop
                        } else {
                            ChunkType::ContentBlockDelta
                        },
                        delta: choice.delta.content.clone(),
                        index: Some(choice.index),
                    });
                }
            }
        }
    }

    Ok(StreamingChunk {
        chunk_type: ChunkType::ContentBlockDelta,
        delta: None,
        index: None,
    })
}

// OpenAI API types

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAITool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<OpenAIContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum OpenAIContent {
    Text(String),
    Parts(Vec<OpenAIContentPart>),
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum OpenAIContentPart {
    Text { text: String },
    ImageUrl { image_url: OpenAIImageUrl },
}

#[derive(Debug, Serialize)]
struct OpenAIImageUrl {
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: OpenAIFunctionCall,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Serialize)]
struct OpenAITool {
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAIFunction,
}

#[derive(Debug, Serialize)]
struct OpenAIFunction {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct OpenAIModelsResponse {
    data: Vec<OpenAIModel>,
}

#[derive(Debug, Deserialize)]
struct OpenAIModel {
    id: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    id: String,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponseMessage {
    content: Option<String>,
    tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u64,
    completion_tokens: u64,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamEvent {
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChoice {
    index: usize,
    delta: OpenAIStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamDelta {
    content: Option<String>,
}

impl From<OpenAIResponse> for CompletionResponse {
    fn from(resp: OpenAIResponse) -> Self {
        let choice = resp.choices.into_iter().next();
        let (content, stop_reason) = match choice {
            Some(c) => {
                let mut blocks = Vec::new();

                if let Some(text) = c.message.content {
                    blocks.push(ContentBlock::Text { text });
                }

                if let Some(tool_calls) = c.message.tool_calls {
                    for tc in tool_calls {
                        let input: serde_json::Value =
                            serde_json::from_str(&tc.function.arguments).unwrap_or_default();
                        blocks.push(ContentBlock::ToolUse {
                            id: tc.id,
                            name: tc.function.name,
                            input,
                        });
                    }
                }

                let stop = c.finish_reason.and_then(|r| match r.as_str() {
                    "stop" => Some(StopReason::EndTurn),
                    "length" => Some(StopReason::MaxTokens),
                    "tool_calls" => Some(StopReason::ToolUse),
                    _ => None,
                });

                (blocks, stop)
            }
            None => (vec![], None),
        };

        Self {
            id: resp.id,
            model: resp.model,
            content,
            stop_reason,
            usage: TokenUsage {
                input_tokens: resp.usage.prompt_tokens,
                output_tokens: resp.usage.completion_tokens,
                cache_read_tokens: None,
                cache_write_tokens: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message;

    #[test]
    fn test_provider_name() {
        let provider = OpenAIProvider::new(ApiKey::new("test".to_string()));
        assert_eq!(provider.name(), "openai");
    }

    #[test]
    fn test_request_conversion() {
        let provider = OpenAIProvider::new(ApiKey::new("test".to_string()));
        let request = CompletionRequest {
            model: "gpt-4o".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: MessageContent::Text("Hello".to_string()),
            }],
            system: Some("You are helpful".to_string()),
            max_tokens: 1024,
            temperature: 0.7,
            stop: None,
            tools: None,
        };

        let openai_req = provider.to_openai_request(&request);
        assert_eq!(openai_req.model, "gpt-4o");
        assert_eq!(openai_req.messages.len(), 2); // system + user
    }
}
