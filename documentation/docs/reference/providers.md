# openclaw-providers API

!!! info "Community Implementation"
    This is a community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw).

AI provider implementations with unified interface.

---

## Installation

```toml
[dependencies]
openclaw-providers = "0.1"
```

---

## Provider Trait

The core trait all providers implement.

```rust
use openclaw_providers::{Provider, MessageRequest, MessageResponse};

#[async_trait]
pub trait Provider: Send + Sync {
    /// Create a message and wait for complete response
    async fn create_message(
        &self,
        request: &MessageRequest,
    ) -> Result<MessageResponse, ProviderError>;

    /// Get provider name
    fn name(&self) -> &str;

    /// Get available models
    fn models(&self) -> &[ModelInfo];
}
```

---

## StreamingProvider Trait

For providers that support streaming.

```rust
use openclaw_providers::{StreamingProvider, StreamChunk};
use futures::Stream;

#[async_trait]
pub trait StreamingProvider: Provider {
    /// Stream a message response
    fn stream_message(
        &self,
        request: &MessageRequest,
    ) -> impl Stream<Item = Result<StreamChunk, ProviderError>> + Send;
}
```

### StreamChunk

```rust
pub enum StreamChunk {
    /// Text content delta
    ContentDelta(String),

    /// Tool use request
    ToolUse(ToolCall),

    /// Token usage update
    Usage(TokenUsage),

    /// Stream complete
    Done,
}
```

---

## AnthropicProvider

Claude models from Anthropic.

### Construction

```rust
use openclaw_providers::AnthropicProvider;

// From API key
let provider = AnthropicProvider::new("sk-ant-...")?;

// With configuration
let provider = AnthropicProvider::builder()
    .api_key("sk-ant-...")
    .base_url("https://api.anthropic.com")
    .default_model("claude-3-5-sonnet-20241022")
    .build()?;

// From environment
let provider = AnthropicProvider::from_env()?;  // Uses ANTHROPIC_API_KEY
```

### Methods

#### create_message

```rust
use openclaw_providers::{MessageRequest, Message, Role};

let request = MessageRequest {
    model: "claude-3-5-sonnet-20241022".into(),
    messages: vec![
        Message {
            role: Role::User,
            content: "Hello, Claude!".into(),
        },
    ],
    max_tokens: 1024,
    system: Some("You are helpful.".into()),
    ..Default::default()
};

let response = provider.create_message(&request).await?;
println!("{}", response.content[0].text);
```

#### stream_message

```rust
use futures::StreamExt;

let stream = provider.stream_message(&request);

tokio::pin!(stream);
while let Some(chunk) = stream.next().await {
    match chunk? {
        StreamChunk::ContentDelta(text) => print!("{}", text),
        StreamChunk::Done => break,
        _ => {}
    }
}
```

### Models

| Model | Context | Description |
|-------|---------|-------------|
| `claude-3-5-sonnet-20241022` | 200K | Latest Sonnet |
| `claude-3-opus-20240229` | 200K | Most capable |
| `claude-3-sonnet-20240229` | 200K | Balanced |
| `claude-3-haiku-20240307` | 200K | Fastest |

---

## OpenAIProvider

GPT models from OpenAI.

### Construction

```rust
use openclaw_providers::OpenAIProvider;

// From API key
let provider = OpenAIProvider::new("sk-...")?;

// With configuration
let provider = OpenAIProvider::builder()
    .api_key("sk-...")
    .org_id("org-...")
    .base_url("https://api.openai.com/v1")
    .default_model("gpt-4o")
    .build()?;

// From environment
let provider = OpenAIProvider::from_env()?;  // Uses OPENAI_API_KEY
```

### Methods

#### create_chat_completion

```rust
use openclaw_providers::{ChatRequest, ChatMessage};

let request = ChatRequest {
    model: "gpt-4o".into(),
    messages: vec![
        ChatMessage {
            role: "system".into(),
            content: "You are helpful.".into(),
        },
        ChatMessage {
            role: "user".into(),
            content: "Hello!".into(),
        },
    ],
    max_tokens: Some(1024),
    ..Default::default()
};

let response = provider.create_chat_completion(&request).await?;
println!("{}", response.choices[0].message.content);
```

#### stream_chat_completion

```rust
use futures::StreamExt;

let stream = provider.stream_chat_completion(&request);

tokio::pin!(stream);
while let Some(chunk) = stream.next().await {
    if let Some(content) = chunk?.choices[0].delta.content.as_ref() {
        print!("{}", content);
    }
}
```

### Models

| Model | Context | Description |
|-------|---------|-------------|
| `gpt-4o` | 128K | Latest GPT-4 Omni |
| `gpt-4o-mini` | 128K | Smaller GPT-4o |
| `gpt-4-turbo` | 128K | GPT-4 Turbo |
| `gpt-4` | 8K | Original GPT-4 |
| `gpt-3.5-turbo` | 16K | Fast, economical |

---

## Request Types

### MessageRequest

For Anthropic Messages API.

```rust
pub struct MessageRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
    pub system: Option<String>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub tools: Option<Vec<Tool>>,
    pub metadata: Option<Value>,
}
```

### ChatRequest

For OpenAI Chat API.

```rust
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stop: Option<Vec<String>>,
    pub functions: Option<Vec<Function>>,
    pub function_call: Option<FunctionCall>,
}
```

---

## Response Types

### MessageResponse

```rust
pub struct MessageResponse {
    pub id: String,
    pub model: String,
    pub content: Vec<ContentBlock>,
    pub stop_reason: StopReason,
    pub usage: TokenUsage,
}

pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
}

pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}
```

### ChatResponse

```rust
pub struct ChatResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

pub struct Choice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
}
```

---

## Error Handling

### ProviderError

```rust
pub enum ProviderError {
    /// Invalid or missing API key
    AuthError { message: String },

    /// Rate limit exceeded
    RateLimited { retry_after: Option<Duration> },

    /// API returned an error
    ApiError { status: u16, body: String },

    /// Network error
    NetworkError(reqwest::Error),

    /// Request validation failed
    ValidationError { field: String, message: String },

    /// Timeout
    Timeout,
}
```

### Error Handling Example

```rust
use openclaw_providers::ProviderError;

match provider.create_message(&request).await {
    Ok(response) => println!("{}", response.content[0].text),
    Err(ProviderError::RateLimited { retry_after }) => {
        if let Some(duration) = retry_after {
            tokio::time::sleep(duration).await;
            // Retry...
        }
    }
    Err(ProviderError::AuthError { message }) => {
        eprintln!("Auth failed: {}", message);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

---

## Feature Flags

| Flag | Description |
|------|-------------|
| `default` | All providers |
| `anthropic` | Anthropic only |
| `openai` | OpenAI only |

```toml
# Only Anthropic
openclaw-providers = { version = "0.1", default-features = false, features = ["anthropic"] }
```

---

## Next Steps

[:material-robot: Agents API](agents.md){ .md-button .md-button--primary }
[:material-nodejs: Node.js API](node.md){ .md-button }
