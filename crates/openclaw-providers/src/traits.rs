//! Provider traits.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::pin::Pin;
use openclaw_core::types::TokenUsage;

/// Provider errors.
#[derive(Error, Debug)]
pub enum ProviderError {
    /// API error.
    #[error("API error: {status} - {message}")]
    Api {
        /// HTTP status code.
        status: u16,
        /// Error message.
        message: String,
    },

    /// Network error.
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Rate limited.
    #[error("Rate limited, retry after {retry_after_secs} seconds")]
    RateLimited {
        /// Seconds to wait before retry.
        retry_after_secs: u64,
    },

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    Config(String),
}

/// Completion request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model to use.
    pub model: String,

    /// Messages in conversation.
    pub messages: Vec<Message>,

    /// System prompt.
    pub system: Option<String>,

    /// Maximum tokens to generate.
    pub max_tokens: u32,

    /// Temperature for sampling.
    pub temperature: f32,

    /// Stop sequences.
    pub stop: Option<Vec<String>>,

    /// Tools available.
    pub tools: Option<Vec<Tool>>,
}

/// A message in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message role.
    pub role: Role,

    /// Message content.
    pub content: MessageContent,
}

/// Message role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// User message.
    User,
    /// Assistant message.
    Assistant,
    /// System message.
    System,
    /// Tool result.
    Tool,
}

/// Message content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Simple text content.
    Text(String),
    /// Structured content blocks.
    Blocks(Vec<ContentBlock>),
}

/// Content block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Text block.
    Text {
        /// Text content.
        text: String,
    },
    /// Image block.
    Image {
        /// Image source.
        source: ImageSource,
    },
    /// Tool use block.
    ToolUse {
        /// Tool ID.
        id: String,
        /// Tool name.
        name: String,
        /// Tool input.
        input: serde_json::Value,
    },
    /// Tool result block.
    ToolResult {
        /// Tool use ID.
        tool_use_id: String,
        /// Tool result content.
        content: String,
        /// Whether tool errored.
        is_error: Option<bool>,
    },
}

/// Image source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    /// Source type.
    #[serde(rename = "type")]
    pub source_type: String,
    /// Media type.
    pub media_type: String,
    /// Base64 data.
    pub data: String,
}

/// Tool definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name.
    pub name: String,
    /// Tool description.
    pub description: String,
    /// Input schema.
    pub input_schema: serde_json::Value,
}

/// Completion response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Response ID.
    pub id: String,

    /// Model used.
    pub model: String,

    /// Response content.
    pub content: Vec<ContentBlock>,

    /// Stop reason.
    pub stop_reason: Option<StopReason>,

    /// Token usage.
    pub usage: TokenUsage,
}

/// Reason the generation stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// End of turn.
    EndTurn,
    /// Hit max tokens.
    MaxTokens,
    /// Hit stop sequence.
    StopSequence,
    /// Tool use requested.
    ToolUse,
}

/// Streaming chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingChunk {
    /// Chunk type.
    pub chunk_type: ChunkType,
    /// Delta content.
    pub delta: Option<String>,
    /// Content block index.
    pub index: Option<usize>,
}

/// Type of streaming chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkType {
    /// Message start.
    MessageStart,
    /// Content block start.
    ContentBlockStart,
    /// Content block delta.
    ContentBlockDelta,
    /// Content block stop.
    ContentBlockStop,
    /// Message delta.
    MessageDelta,
    /// Message stop.
    MessageStop,
}

/// AI provider trait.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Provider name.
    fn name(&self) -> &str;

    /// List available models.
    async fn list_models(&self) -> Result<Vec<String>, ProviderError>;

    /// Create a completion.
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError>;

    /// Create a streaming completion.
    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<StreamingChunk, ProviderError>> + Send>>, ProviderError>;
}
