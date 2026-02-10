//! Anthropic Claude API provider.

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

const ANTHROPIC_VERSION: &str = "2023-06-01";
const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";

/// Anthropic API provider.
pub struct AnthropicProvider {
    client: Client,
    api_key: ApiKey,
    base_url: String,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider.
    #[must_use]
    pub fn new(api_key: ApiKey) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    /// Create with custom base URL.
    #[must_use]
    pub fn with_base_url(api_key: ApiKey, base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: base_url.into(),
        }
    }

    /// Convert our request format to Anthropic's API format.
    fn to_anthropic_request(&self, request: &CompletionRequest) -> AnthropicRequest {
        let messages: Vec<AnthropicMessage> = request
            .messages
            .iter()
            .filter(|m| m.role != Role::System) // System handled separately
            .map(|m| AnthropicMessage {
                role: match m.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::Tool => "user".to_string(), // Tool results come from user
                    Role::System => unreachable!(),
                },
                content: match &m.content {
                    MessageContent::Text(text) => AnthropicContent::Text(text.clone()),
                    MessageContent::Blocks(blocks) => {
                        AnthropicContent::Blocks(blocks.iter().map(|b| b.clone().into()).collect())
                    }
                },
            })
            .collect();

        let tools = request.tools.as_ref().map(|tools| {
            tools
                .iter()
                .map(|t| AnthropicTool {
                    name: t.name.clone(),
                    description: t.description.clone(),
                    input_schema: t.input_schema.clone(),
                })
                .collect()
        });

        AnthropicRequest {
            model: request.model.clone(),
            messages,
            system: request.system.clone(),
            max_tokens: request.max_tokens,
            temperature: Some(request.temperature),
            stop_sequences: request.stop.clone(),
            tools,
            stream: Some(false),
        }
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    async fn list_models(&self) -> Result<Vec<String>, ProviderError> {
        Ok(vec![
            "claude-sonnet-4-20250514".to_string(),
            "claude-opus-4-20250514".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-5-haiku-20241022".to_string(),
            "claude-3-opus-20240229".to_string(),
        ])
    }

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError> {
        let url = format!("{}/v1/messages", self.base_url);
        let anthropic_request = self.to_anthropic_request(&request);

        let response = self
            .client
            .post(&url)
            .header("x-api-key", self.api_key.expose())
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&anthropic_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();

            // Check for rate limiting
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

        let result: AnthropicResponse = response.json().await?;
        Ok(result.into())
    }

    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<
        Pin<Box<dyn Stream<Item = Result<StreamingChunk, ProviderError>> + Send>>,
        ProviderError,
    > {
        let url = format!("{}/v1/messages", self.base_url);
        let mut anthropic_request = self.to_anthropic_request(&request);
        anthropic_request.stream = Some(true);

        let response = self
            .client
            .post(&url)
            .header("x-api-key", self.api_key.expose())
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&anthropic_request)
            .send()
            .await?;

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

/// Parse SSE event from Anthropic streaming response.
fn parse_sse_event(text: &str) -> Result<StreamingChunk, ProviderError> {
    // SSE format: "event: <type>\ndata: <json>\n\n"
    for line in text.lines() {
        if let Some(data) = line.strip_prefix("data: ") {
            if data == "[DONE]" {
                return Ok(StreamingChunk {
                    chunk_type: ChunkType::MessageStop,
                    delta: None,
                    index: None,
                });
            }

            if let Ok(event) = serde_json::from_str::<AnthropicStreamEvent>(data) {
                return Ok(event.into());
            }
        }
    }

    // No parseable event, return empty delta
    Ok(StreamingChunk {
        chunk_type: ChunkType::ContentBlockDelta,
        delta: None,
        index: None,
    })
}

// Anthropic API types

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<AnthropicTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: AnthropicContent,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum AnthropicContent {
    Text(String),
    Blocks(Vec<AnthropicContentBlock>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicContentBlock {
    Text {
        text: String,
    },
    Image {
        source: ImageSourceApi,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct ImageSourceApi {
    #[serde(rename = "type")]
    source_type: String,
    media_type: String,
    data: String,
}

#[derive(Debug, Serialize)]
struct AnthropicTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    id: String,
    model: String,
    content: Vec<AnthropicContentBlock>,
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
    #[serde(default)]
    cache_read_input_tokens: Option<u32>,
    #[serde(default)]
    cache_creation_input_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    delta: Option<AnthropicDelta>,
    #[serde(default)]
    index: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct AnthropicDelta {
    #[serde(rename = "type")]
    delta_type: Option<String>,
    text: Option<String>,
}

// Conversions

impl From<ContentBlock> for AnthropicContentBlock {
    fn from(block: ContentBlock) -> Self {
        match block {
            ContentBlock::Text { text } => AnthropicContentBlock::Text { text },
            ContentBlock::Image { source } => AnthropicContentBlock::Image {
                source: ImageSourceApi {
                    source_type: source.source_type,
                    media_type: source.media_type,
                    data: source.data,
                },
            },
            ContentBlock::ToolUse { id, name, input } => {
                AnthropicContentBlock::ToolUse { id, name, input }
            }
            ContentBlock::ToolResult {
                tool_use_id,
                content,
                ..
            } => AnthropicContentBlock::ToolResult {
                tool_use_id,
                content,
            },
        }
    }
}

impl From<AnthropicContentBlock> for ContentBlock {
    fn from(block: AnthropicContentBlock) -> Self {
        match block {
            AnthropicContentBlock::Text { text } => ContentBlock::Text { text },
            AnthropicContentBlock::Image { source } => ContentBlock::Image {
                source: crate::traits::ImageSource {
                    source_type: source.source_type,
                    media_type: source.media_type,
                    data: source.data,
                },
            },
            AnthropicContentBlock::ToolUse { id, name, input } => {
                ContentBlock::ToolUse { id, name, input }
            }
            AnthropicContentBlock::ToolResult {
                tool_use_id,
                content,
            } => ContentBlock::ToolResult {
                tool_use_id,
                content,
                is_error: None,
            },
        }
    }
}

impl From<AnthropicResponse> for CompletionResponse {
    fn from(resp: AnthropicResponse) -> Self {
        Self {
            id: resp.id,
            model: resp.model,
            content: resp.content.into_iter().map(Into::into).collect(),
            stop_reason: resp.stop_reason.and_then(|s| match s.as_str() {
                "end_turn" => Some(StopReason::EndTurn),
                "max_tokens" => Some(StopReason::MaxTokens),
                "stop_sequence" => Some(StopReason::StopSequence),
                "tool_use" => Some(StopReason::ToolUse),
                _ => None,
            }),
            usage: TokenUsage {
                input_tokens: u64::from(resp.usage.input_tokens),
                output_tokens: u64::from(resp.usage.output_tokens),
                cache_read_tokens: resp.usage.cache_read_input_tokens.map(u64::from),
                cache_write_tokens: resp.usage.cache_creation_input_tokens.map(u64::from),
            },
        }
    }
}

impl From<AnthropicStreamEvent> for StreamingChunk {
    fn from(event: AnthropicStreamEvent) -> Self {
        let chunk_type = match event.event_type.as_str() {
            "message_start" => ChunkType::MessageStart,
            "content_block_start" => ChunkType::ContentBlockStart,
            "content_block_delta" => ChunkType::ContentBlockDelta,
            "content_block_stop" => ChunkType::ContentBlockStop,
            "message_delta" => ChunkType::MessageDelta,
            "message_stop" => ChunkType::MessageStop,
            _ => ChunkType::ContentBlockDelta,
        };

        Self {
            chunk_type,
            delta: event.delta.and_then(|d| d.text),
            index: event.index,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message;

    #[test]
    fn test_provider_name() {
        let provider = AnthropicProvider::new(ApiKey::new("test".to_string()));
        assert_eq!(provider.name(), "anthropic");
    }

    #[test]
    fn test_request_conversion() {
        let provider = AnthropicProvider::new(ApiKey::new("test".to_string()));
        let request = CompletionRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
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

        let anthropic_req = provider.to_anthropic_request(&request);
        assert_eq!(anthropic_req.model, "claude-3-5-sonnet-20241022");
        assert_eq!(anthropic_req.messages.len(), 1);
        assert_eq!(anthropic_req.system, Some("You are helpful".to_string()));
    }
}
