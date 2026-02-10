# openclaw-agents API

!!! info "Community Implementation"
    This is a community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw).

Agent runtime for managing AI conversations with tool support.

---

## Installation

```toml
[dependencies]
openclaw-agents = "0.1"
```

---

## AgentRuntime

The main agent runtime.

### Construction

```rust
use openclaw_agents::{AgentRuntime, AgentConfig};

// From configuration
let config = AgentConfig {
    provider: "anthropic".into(),
    model: "claude-3-5-sonnet-20241022".into(),
    system_prompt: Some("You are a helpful assistant.".into()),
    max_tokens: 4096,
    temperature: 0.7,
    tools: vec!["file_read".into(), "bash".into()],
};

let runtime = AgentRuntime::new(config)?;
```

### Builder Pattern

```rust
let runtime = AgentRuntime::builder()
    .provider("anthropic")
    .model("claude-3-5-sonnet-20241022")
    .system_prompt("You are helpful.")
    .max_tokens(4096)
    .temperature(0.7)
    .tool("file_read")
    .tool("bash")
    .build()?;
```

### Methods

#### chat

Send a message and get a complete response.

```rust
let response = runtime.chat("Hello!").await?;
println!("{}", response.content);
```

#### chat_with_history

Send with conversation history.

```rust
use openclaw_core::types::{Message, Role};

let history = vec![
    Message {
        role: Role::User,
        content: "What's 2+2?".into(),
    },
    Message {
        role: Role::Assistant,
        content: "2+2 equals 4.".into(),
    },
];

let response = runtime.chat_with_history("And 3+3?", &history).await?;
```

#### stream_chat

Stream a response.

```rust
use futures::StreamExt;

let mut stream = runtime.stream_chat("Tell me a story").await;

while let Some(chunk) = stream.next().await {
    match chunk? {
        AgentChunk::Text(text) => print!("{}", text),
        AgentChunk::ToolCall(call) => println!("\n[Tool: {}]", call.name),
        AgentChunk::ToolResult(result) => println!("[Result: {}]", result.content),
        AgentChunk::Done => break,
    }
}
```

---

## Session

Manages a conversation session.

### Creation

```rust
use openclaw_agents::Session;

let session = Session::new(runtime);
```

### Methods

#### send

Send a message in the session context.

```rust
let response = session.send("Hello!").await?;
println!("{}", response.content);

// History is maintained
let response = session.send("What did I just say?").await?;
```

#### history

Get conversation history.

```rust
let messages = session.history();
for msg in messages {
    println!("{:?}: {}", msg.role, msg.content);
}
```

#### clear

Clear conversation history.

```rust
session.clear();
```

#### save / load

Persist sessions.

```rust
// Save to file
session.save("session.json")?;

// Load from file
let session = Session::load("session.json")?;
```

---

## ToolExecutor

Executes tools in a sandboxed environment.

### Built-in Tools

```rust
use openclaw_agents::tools::{FileReadTool, FileWriteTool, BashTool};

let file_read = FileReadTool::new("/workspace");
let bash = BashTool::new().sandbox(true);
```

### Custom Tools

```rust
use openclaw_agents::{ToolExecutor, ToolCall, ToolResult};
use async_trait::async_trait;

struct MyTool;

#[async_trait]
impl ToolExecutor for MyTool {
    fn name(&self) -> &str {
        "my_tool"
    }

    fn description(&self) -> &str {
        "Does something useful"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "input": { "type": "string" }
            },
            "required": ["input"]
        })
    }

    async fn execute(&self, call: &ToolCall) -> Result<ToolResult, ToolError> {
        let input = call.arguments["input"].as_str().unwrap();
        Ok(ToolResult {
            tool_use_id: call.id.clone(),
            content: format!("Processed: {}", input).into(),
            is_error: false,
        })
    }
}
```

### Registering Tools

```rust
let runtime = AgentRuntime::builder()
    .provider("anthropic")
    .model("claude-3-5-sonnet-20241022")
    .register_tool(Box::new(MyTool))
    .register_tool(Box::new(FileReadTool::new("/workspace")))
    .build()?;
```

---

## AgentConfig

Configuration for an agent.

```rust
pub struct AgentConfig {
    /// Provider name
    pub provider: String,

    /// Model identifier
    pub model: String,

    /// System prompt
    pub system_prompt: Option<String>,

    /// Maximum tokens per response
    pub max_tokens: u32,

    /// Sampling temperature (0-1)
    pub temperature: f32,

    /// Top-p sampling
    pub top_p: Option<f32>,

    /// Enabled tool names
    pub tools: Vec<String>,

    /// Stop sequences
    pub stop_sequences: Vec<String>,

    /// Custom metadata
    pub metadata: HashMap<String, Value>,
}
```

---

## Response Types

### AgentResponse

```rust
pub struct AgentResponse {
    /// Response content
    pub content: String,

    /// Tool calls made
    pub tool_calls: Vec<ToolCall>,

    /// Stop reason
    pub stop_reason: StopReason,

    /// Token usage
    pub usage: TokenUsage,

    /// Response metadata
    pub metadata: HashMap<String, Value>,
}
```

### AgentChunk

For streaming responses.

```rust
pub enum AgentChunk {
    /// Text content
    Text(String),

    /// Tool call request
    ToolCall(ToolCall),

    /// Tool execution result
    ToolResult(ToolResult),

    /// Stream complete
    Done,
}
```

---

## Error Handling

### AgentError

```rust
pub enum AgentError {
    /// Provider error
    ProviderError(ProviderError),

    /// Tool execution error
    ToolError { tool: String, message: String },

    /// Configuration error
    ConfigError { message: String },

    /// Context length exceeded
    ContextLengthExceeded { max: u32, current: u32 },

    /// Session error
    SessionError { message: String },
}
```

### Example

```rust
use openclaw_agents::AgentError;

match runtime.chat("Hello").await {
    Ok(response) => println!("{}", response.content),
    Err(AgentError::ToolError { tool, message }) => {
        eprintln!("Tool {} failed: {}", tool, message);
    }
    Err(AgentError::ContextLengthExceeded { max, current }) => {
        eprintln!("Context too long: {} > {}", current, max);
        // Clear history and retry
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

---

## Events

Subscribe to agent events.

```rust
use openclaw_agents::AgentEvent;

runtime.on_event(|event| {
    match event {
        AgentEvent::MessageSent(msg) => println!("Sent: {:?}", msg),
        AgentEvent::MessageReceived(msg) => println!("Received: {:?}", msg),
        AgentEvent::ToolCalled(call) => println!("Tool: {}", call.name),
        AgentEvent::ToolCompleted(result) => println!("Result: {:?}", result),
    }
});
```

---

## Context Management

### Token Counting

```rust
let token_count = runtime.count_tokens(&messages)?;
println!("Tokens: {}", token_count);
```

### Context Window

```rust
let remaining = runtime.remaining_context()?;
println!("Remaining tokens: {}", remaining);
```

### Auto-Summarization

```rust
let runtime = AgentRuntime::builder()
    .provider("anthropic")
    .model("claude-3-5-sonnet-20241022")
    .auto_summarize(true)
    .summarize_threshold(50000)  // Summarize at 50k tokens
    .build()?;
```

---

## Next Steps

[:material-nodejs: Node.js API](node.md){ .md-button .md-button--primary }
[:material-console: CLI Reference](cli-commands.md){ .md-button }
