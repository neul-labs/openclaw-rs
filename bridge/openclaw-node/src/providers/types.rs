//! JavaScript-friendly types for provider bindings.

use napi_derive::napi;
use serde::{Deserialize, Serialize};

use openclaw_providers::{
    CompletionRequest, CompletionResponse, ContentBlock, Message, MessageContent, Role, StopReason,
    Tool as ProviderTool,
};

/// A message in a conversation.
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsMessage {
    /// Role: "user", "assistant", "system", or "tool"
    pub role: String,
    /// Message content (text)
    pub content: String,
    /// Tool use ID (for tool results)
    pub tool_use_id: Option<String>,
    /// Tool name (for tool results)
    pub tool_name: Option<String>,
}

/// A completion request.
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsCompletionRequest {
    /// Model to use (e.g., "claude-3-5-sonnet-20241022")
    pub model: String,
    /// Conversation messages
    pub messages: Vec<JsMessage>,
    /// System prompt (optional)
    pub system: Option<String>,
    /// Maximum tokens to generate
    pub max_tokens: u32,
    /// Temperature (0.0-1.0)
    pub temperature: Option<f64>,
    /// Stop sequences
    pub stop: Option<Vec<String>>,
    /// Tools available to the model
    pub tools: Option<Vec<JsTool>>,
}

/// A tool definition.
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsTool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// JSON schema for input parameters
    pub input_schema: serde_json::Value,
}

/// A completion response.
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsCompletionResponse {
    /// Response ID
    pub id: String,
    /// Model used
    pub model: String,
    /// Text content (joined from all text blocks)
    pub content: String,
    /// Stop reason: "`end_turn`", "`max_tokens`", "`stop_sequence`", "`tool_use`"
    pub stop_reason: Option<String>,
    /// Tool calls made by the model
    pub tool_calls: Option<Vec<JsToolCall>>,
    /// Token usage
    pub usage: JsTokenUsage,
}

/// A tool call from the model.
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsToolCall {
    /// Tool use ID (for providing results back)
    pub id: String,
    /// Tool name
    pub name: String,
    /// Tool input parameters
    pub input: serde_json::Value,
}

/// Token usage information.
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsTokenUsage {
    /// Input tokens consumed
    pub input_tokens: u32,
    /// Output tokens generated
    pub output_tokens: u32,
    /// Tokens read from cache (if applicable)
    pub cache_read_tokens: Option<u32>,
    /// Tokens written to cache (if applicable)
    pub cache_write_tokens: Option<u32>,
}

/// A streaming chunk.
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsStreamChunk {
    /// Chunk type: "`message_start`", "`content_block_start`", "`content_block_delta`",
    /// "`content_block_stop`", "`message_delta`", "`message_stop`"
    pub chunk_type: String,
    /// Text delta (for `content_block_delta`)
    pub delta: Option<String>,
    /// Block index
    pub index: Option<u32>,
    /// Stop reason (for `message_delta` with stop)
    pub stop_reason: Option<String>,
}

// ---- Conversion functions ----

/// Convert `JsMessage` to internal Message.
#[must_use]
pub fn convert_js_message(msg: &JsMessage) -> Message {
    let role = match msg.role.as_str() {
        "user" => Role::User,
        "assistant" => Role::Assistant,
        "system" => Role::System,
        "tool" => Role::Tool,
        _ => Role::User,
    };

    // Handle tool results
    if role == Role::Tool {
        Message {
            role,
            content: MessageContent::Blocks(vec![ContentBlock::ToolResult {
                tool_use_id: msg.tool_use_id.clone().unwrap_or_default(),
                content: msg.content.clone(),
                is_error: None,
            }]),
        }
    } else {
        Message {
            role,
            content: MessageContent::Text(msg.content.clone()),
        }
    }
}

/// Convert `JsTool` to internal Tool.
#[must_use]
pub fn convert_js_tool(tool: &JsTool) -> ProviderTool {
    ProviderTool {
        name: tool.name.clone(),
        description: tool.description.clone(),
        input_schema: tool.input_schema.clone(),
    }
}

/// Convert `JsCompletionRequest` to internal `CompletionRequest`.
pub fn convert_request(req: JsCompletionRequest) -> CompletionRequest {
    CompletionRequest {
        model: req.model,
        messages: req.messages.iter().map(convert_js_message).collect(),
        system: req.system,
        max_tokens: req.max_tokens,
        temperature: req.temperature.map_or(1.0, |t| t as f32),
        stop: req.stop,
        tools: req
            .tools
            .map(|tools| tools.iter().map(convert_js_tool).collect()),
    }
}

/// Convert internal `CompletionResponse` to `JsCompletionResponse`.
#[must_use]
pub fn convert_response(resp: CompletionResponse) -> JsCompletionResponse {
    // Extract text content
    let content = resp
        .content
        .iter()
        .filter_map(|block| {
            if let ContentBlock::Text { text } = block {
                Some(text.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("");

    // Extract tool calls
    let tool_calls: Vec<JsToolCall> = resp
        .content
        .iter()
        .filter_map(|block| {
            if let ContentBlock::ToolUse { id, name, input } = block {
                Some(JsToolCall {
                    id: id.clone(),
                    name: name.clone(),
                    input: input.clone(),
                })
            } else {
                None
            }
        })
        .collect();

    let stop_reason = resp.stop_reason.map(|sr| match sr {
        StopReason::EndTurn => "end_turn".to_string(),
        StopReason::MaxTokens => "max_tokens".to_string(),
        StopReason::StopSequence => "stop_sequence".to_string(),
        StopReason::ToolUse => "tool_use".to_string(),
    });

    JsCompletionResponse {
        id: resp.id,
        model: resp.model,
        content,
        stop_reason,
        tool_calls: if tool_calls.is_empty() {
            None
        } else {
            Some(tool_calls)
        },
        usage: JsTokenUsage {
            input_tokens: resp.usage.input_tokens as u32,
            output_tokens: resp.usage.output_tokens as u32,
            cache_read_tokens: resp.usage.cache_read_tokens.map(|v| v as u32),
            cache_write_tokens: resp.usage.cache_write_tokens.map(|v| v as u32),
        },
    }
}
