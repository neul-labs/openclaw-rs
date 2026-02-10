# openclaw-core API

!!! info "Community Implementation"
    A Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw) by [Neul Labs](https://neullabs.com).

Core types, authentication, and configuration for openclaw-rs.

---

## Installation

```toml
[dependencies]
openclaw-core = "0.1"
```

---

## Types Module

### Message

Represents a chat message.

```rust
use openclaw_core::types::{Message, Role, Content};

let message = Message {
    role: Role::User,
    content: Content::Text("Hello, Claude!".into()),
    metadata: None,
};
```

#### Fields

| Field | Type | Description |
|-------|------|-------------|
| `role` | `Role` | Message sender role |
| `content` | `Content` | Message content |
| `metadata` | `Option<Metadata>` | Optional metadata |

### Role

Message sender enumeration.

```rust
pub enum Role {
    User,
    Assistant,
    System,
}
```

### Content

Message content variants.

```rust
pub enum Content {
    Text(String),
    Image(ImageData),
    ToolUse(ToolCall),
    ToolResult(ToolResult),
    Mixed(Vec<ContentBlock>),
}
```

### Tool

Tool definition.

```rust
use openclaw_core::types::Tool;

let tool = Tool {
    name: "read_file".into(),
    description: "Read contents of a file".into(),
    input_schema: serde_json::json!({
        "type": "object",
        "properties": {
            "path": { "type": "string" }
        },
        "required": ["path"]
    }),
};
```

#### Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Tool identifier |
| `description` | `String` | Human-readable description |
| `input_schema` | `Value` | JSON Schema for inputs |

### ToolCall

Request to execute a tool.

```rust
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}
```

### ToolResult

Result from tool execution.

```rust
pub struct ToolResult {
    pub tool_use_id: String,
    pub content: Content,
    pub is_error: bool,
}
```

---

## Auth Module

### CredentialStore

Encrypted credential storage.

```rust
use openclaw_core::auth::CredentialStore;

let store = CredentialStore::new("~/.openclaw/credentials")?;

// Store a credential
store.set("anthropic", "sk-ant-...")?;

// Retrieve a credential
let key = store.get("anthropic")?;

// Delete a credential
store.delete("anthropic")?;
```

#### Methods

| Method | Description |
|--------|-------------|
| `new(path)` | Create store at path |
| `set(name, value)` | Store encrypted credential |
| `get(name)` | Retrieve credential |
| `delete(name)` | Remove credential |
| `list()` | List stored credential names |
| `exists(name)` | Check if credential exists |

### AuthService

High-level authentication management.

```rust
use openclaw_core::auth::AuthService;

let auth = AuthService::new()?;

// Configure a provider
auth.configure_provider("anthropic", ProviderAuth {
    api_key: Some("sk-ant-...".into()),
    api_key_env: None,
})?;

// Check configuration
if auth.is_provider_configured("anthropic")? {
    let creds = auth.get_provider_credentials("anthropic")?;
}
```

#### Methods

| Method | Description |
|--------|-------------|
| `new()` | Create with default config |
| `configure_provider(name, auth)` | Set provider credentials |
| `is_provider_configured(name)` | Check if configured |
| `get_provider_credentials(name)` | Get credentials |
| `clear_provider(name)` | Remove provider config |

---

## Config Module

### Config

Application configuration.

```rust
use openclaw_core::config::Config;

// Load from default path
let config = Config::load()?;

// Load from specific path
let config = Config::load_from("path/to/config.json")?;

// Access values
let port = config.gateway.port;
let provider = &config.providers["anthropic"];
```

#### Structure

```rust
pub struct Config {
    pub gateway: GatewayConfig,
    pub providers: HashMap<String, ProviderConfig>,
    pub agents: HashMap<String, AgentConfig>,
    pub workspace: WorkspaceConfig,
}

pub struct GatewayConfig {
    pub port: u16,
    pub bind: String,
    pub cors_origins: Vec<String>,
    pub tls: Option<TlsConfig>,
}

pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub api_key_env: Option<String>,
    pub base_url: Option<String>,
    pub default_model: Option<String>,
}

pub struct AgentConfig {
    pub provider: String,
    pub model: String,
    pub system_prompt: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub tools: Vec<String>,
}
```

#### Methods

| Method | Description |
|--------|-------------|
| `load()` | Load from default path |
| `load_from(path)` | Load from specific path |
| `save()` | Save to default path |
| `save_to(path)` | Save to specific path |
| `validate()` | Validate configuration |

---

## Events Module

### Event

Base event structure.

```rust
use openclaw_core::events::{Event, EventType};

pub struct Event {
    pub id: String,
    pub event_type: EventType,
    pub payload: Value,
    pub timestamp: DateTime<Utc>,
}
```

### EventType

Event type enumeration.

```rust
pub enum EventType {
    MessageReceived,
    MessageSent,
    ToolCalled,
    ToolCompleted,
    SessionStarted,
    SessionEnded,
    Error,
}
```

### EventBus

Publish/subscribe event bus.

```rust
use openclaw_core::events::EventBus;

let bus = EventBus::new();

// Subscribe
let subscription = bus.subscribe(|event| {
    println!("Received: {:?}", event);
});

// Publish
bus.publish(Event {
    id: uuid::Uuid::new_v4().to_string(),
    event_type: EventType::MessageReceived,
    payload: serde_json::json!({ "text": "Hello" }),
    timestamp: Utc::now(),
});

// Unsubscribe
drop(subscription);
```

---

## Error Types

### CoreError

Main error type.

```rust
use openclaw_core::error::CoreError;

pub enum CoreError {
    ConfigError(ConfigError),
    AuthError(AuthError),
    IoError(std::io::Error),
    SerdeError(serde_json::Error),
}
```

### Result Type

```rust
pub type Result<T> = std::result::Result<T, CoreError>;
```

---

## Feature Flags

| Flag | Description |
|------|-------------|
| `default` | All features |
| `auth` | Credential storage |
| `config` | Configuration loading |
| `events` | Event system |

```toml
# Minimal
openclaw-core = { version = "0.1", default-features = false }

# Only auth
openclaw-core = { version = "0.1", default-features = false, features = ["auth"] }
```

---

## Next Steps

[:material-cloud: Providers API](providers.md){ .md-button .md-button--primary }
[:material-robot: Agents API](agents.md){ .md-button }
