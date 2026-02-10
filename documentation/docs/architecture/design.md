# Design Principles

!!! info "Community Implementation"
    A Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw) by [Neul Labs](https://neullabs.com).

The design of openclaw-rs follows specific principles to ensure reliability, performance, and maintainability.

---

## Core Principles

### 1. Type Safety First

Rust's type system is leveraged extensively:

```rust
// Strong typing prevents runtime errors
pub struct Message {
    pub role: Role,           // Enum, not string
    pub content: Content,     // Structured content
    pub metadata: Metadata,   // Typed metadata
}

pub enum Role {
    User,
    Assistant,
    System,
}

pub enum Content {
    Text(String),
    Image(ImageData),
    ToolUse(ToolCall),
    ToolResult(ToolResult),
}
```

Benefits:

- Compile-time error detection
- Self-documenting APIs
- Impossible states are unrepresentable

---

### 2. Async by Default

All I/O operations are async using Tokio:

```rust
// Non-blocking provider calls
impl Provider for AnthropicProvider {
    async fn create_message(&self, request: &Request) -> Result<Response> {
        // Async HTTP request
        let response = self.client
            .post(&self.endpoint)
            .json(request)
            .send()
            .await?;

        Ok(response.json().await?)
    }
}
```

Benefits:

- High concurrency with low resource usage
- Efficient streaming
- Scalable to many simultaneous connections

---

### 3. Zero-Cost Abstractions

Abstractions don't incur runtime overhead:

```rust
// Trait provides abstraction
pub trait Provider: Send + Sync {
    async fn create_message(&self, request: &Request) -> Result<Response>;
}

// Concrete implementation - no virtual dispatch overhead when type is known
pub struct AnthropicProvider { /* ... */ }
pub struct OpenAIProvider { /* ... */ }
```

Benefits:

- Clean APIs without performance penalty
- Compile-time polymorphism where possible
- Dynamic dispatch only when needed

---

### 4. Fail-Fast with Rich Errors

Errors are explicit and informative:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Authentication failed: {message}")]
    AuthError { message: String },

    #[error("Rate limited: retry after {retry_after:?}")]
    RateLimited { retry_after: Option<Duration> },

    #[error("API error: {status} - {body}")]
    ApiError { status: u16, body: String },

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
}
```

Benefits:

- No silent failures
- Actionable error messages
- Error recovery information

---

### 5. Configuration as Code

Configuration is type-checked:

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub gateway: GatewayConfig,
    pub providers: HashMap<String, ProviderConfig>,
    pub agents: HashMap<String, AgentConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GatewayConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_bind")]
    pub bind: String,
}
```

Benefits:

- Invalid config rejected at load time
- Default values are explicit
- Schema is self-documenting

---

### 6. Streaming First

Streaming is a core consideration, not an afterthought:

```rust
pub trait StreamingProvider: Provider {
    fn stream_message(
        &self,
        request: &Request,
    ) -> impl Stream<Item = Result<StreamChunk>> + Send;
}

pub enum StreamChunk {
    ContentDelta(String),
    ToolUse(ToolCall),
    Usage(TokenUsage),
    Done,
}
```

Benefits:

- Real-time responses
- Memory efficient for long outputs
- Better user experience

---

### 7. Layered Architecture

Clear separation of concerns:

```
┌─────────────────────────────────────┐
│          Application Layer          │  CLI, Gateway, UI
├─────────────────────────────────────┤
│           Service Layer             │  Agents, Sessions
├─────────────────────────────────────┤
│         Integration Layer           │  Providers, Plugins
├─────────────────────────────────────┤
│            Core Layer               │  Types, Auth, Config
└─────────────────────────────────────┘
```

Benefits:

- Independent testing
- Clear dependencies
- Easier refactoring

---

### 8. Trait-Based Interfaces

Interfaces defined as traits enable flexibility:

```rust
// Core trait
pub trait ToolExecutor: Send + Sync {
    async fn execute(&self, call: &ToolCall) -> Result<ToolResult>;
    fn capabilities(&self) -> &[ToolCapability];
}

// Multiple implementations
pub struct SandboxedExecutor { /* ... */ }
pub struct DirectExecutor { /* ... */ }
pub struct RemoteExecutor { /* ... */ }
```

Benefits:

- Swappable implementations
- Easy mocking for tests
- Clear contracts

---

## Architectural Decisions

### Why Axum?

Chosen for the gateway:

- Built on Tokio/Tower ecosystem
- Excellent WebSocket support
- Type-safe extractors
- Middleware composition

### Why Pinia over Vuex?

For the web UI:

- Better TypeScript support
- Simpler API
- Composition API friendly
- Smaller bundle size

### Why JSON-RPC over REST?

For the API:

- Better for streaming
- Consistent request/response format
- Easier batching
- Method-based routing

### Why napi-rs?

For Node.js bindings:

- Best performance
- Full async support
- Good TypeScript integration
- Active maintenance

---

## Trade-offs

### Compile Time vs Runtime Safety

**Choice**: Maximum compile-time safety

Trade-off: Longer compile times, steeper learning curve

Benefit: Fewer runtime bugs, better refactoring

### Flexibility vs Simplicity

**Choice**: Favor simplicity

Trade-off: Less configurability in some areas

Benefit: Easier to understand and maintain

### Performance vs Compatibility

**Choice**: Native Rust performance

Trade-off: Requires Rust toolchain for development

Benefit: Excellent runtime performance

---

## Next Steps

[:material-shield: Security Model](security.md){ .md-button .md-button--primary }
[:material-api: API Reference](../reference/core.md){ .md-button }
