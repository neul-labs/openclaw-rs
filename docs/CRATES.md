# Crates

Detailed documentation for each crate in the OpenClaw Rust Core workspace.

## Dependency Graph

```
openclaw-cli
    ├── openclaw-gateway
    │   ├── openclaw-agents
    │   │   ├── openclaw-providers
    │   │   │   └── openclaw-core
    │   │   └── openclaw-core
    │   ├── openclaw-channels
    │   │   └── openclaw-core
    │   └── openclaw-core
    ├── openclaw-core
    └── openclaw-agents

openclaw-plugins
    ├── openclaw-core
    └── openclaw-ipc

openclaw-node (napi bindings)
    ├── openclaw-core
    ├── openclaw-providers
    └── openclaw-agents
```

---

## openclaw-core

Foundation crate with types, configuration, events, and security primitives.

### Modules

| Module | Description |
|--------|-------------|
| `types` | Core type definitions |
| `config` | JSON5 configuration loading |
| `events` | Event store and projections |
| `secrets` | Credential encryption |
| `auth` | Authentication management |
| `validation` | Input validation |

### Key Types

```rust
// Identifiers
pub struct AgentId(String);
pub struct ChannelId(String);
pub struct SessionKey(String);
pub struct PeerId(String);

// Messages
pub struct Message {
    pub id: String,
    pub content: String,
    pub attachments: Vec<Attachment>,
    pub timestamp: DateTime<Utc>,
}

// Events
pub enum SessionEventKind {
    SessionStarted { channel: String, peer_id: String },
    MessageReceived { content: String, attachments: Vec<AttachmentMeta> },
    MessageSent { content: String, message_id: String },
    ToolCalled { tool_name: String, params: Value },
    ToolResult { tool_name: String, result: Value, success: bool },
    AgentResponse { content: String, model: String, tokens: TokenUsage },
    SessionEnded { reason: String },
    StateChanged { key: String, value: Value },
}
```

### Usage

```rust
use openclaw_core::{Config, EventStore, SessionEvent, SessionEventKind};

// Load configuration
let config = Config::load()?;

// Create event store
let store = EventStore::open(Path::new("~/.openclaw/sessions"))?;

// Append event
let event = SessionEvent::new(
    session_key,
    "default".to_string(),
    SessionEventKind::MessageReceived {
        content: "Hello".to_string(),
        attachments: vec![],
    },
);
store.append(&event)?;

// Get projection
let projection = store.get_projection(&session_key)?;
```

---

## openclaw-ipc

Inter-process communication for TypeScript plugin bridge.

### Modules

| Module | Description |
|--------|-------------|
| `messages` | IPC message types |
| `transport` | nng socket transport |

### Message Types

```rust
pub enum IpcMessage {
    Request(IpcRequest),
    Response(IpcResponse),
    Event(IpcEvent),
}

pub struct IpcRequest {
    pub id: String,
    pub method: String,
    pub params: Value,
}

pub enum IpcEvent {
    MessageReceived { session_key: String, content: String },
    ToolCalled { session_key: String, tool_name: String },
    SessionEnded { session_key: String, reason: String },
}
```

### Usage

```rust
use openclaw_ipc::{IpcTransport, IpcRequest};

let transport = IpcTransport::connect("ipc:///tmp/openclaw.sock").await?;

let request = IpcRequest {
    id: "1".to_string(),
    method: "onMessage".to_string(),
    params: json!({ "content": "Hello" }),
};

let response = transport.send(request).await?;
```

---

## openclaw-providers

AI provider clients (Anthropic, OpenAI, etc.).

### Modules

| Module | Description |
|--------|-------------|
| `traits` | Provider trait definitions |
| `anthropic` | Anthropic Claude client (full API + SSE streaming) |
| `openai` | OpenAI GPT client (full API + SSE streaming) |
| `usage` | Token usage tracking |

### Provider Trait

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError>;
    async fn complete_stream(&self, request: CompletionRequest) -> Result<BoxStream<StreamingChunk>, ProviderError>;
    fn name(&self) -> &str;
    fn default_model(&self) -> &str;
}

pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub system: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub tools: Vec<Tool>,
    pub stop_sequences: Vec<String>,
}
```

### Usage

```rust
use openclaw_providers::{AnthropicProvider, Provider, CompletionRequest, Message, Role};
use openclaw_core::ApiKey;

let provider = AnthropicProvider::new(ApiKey::new("sk-...".to_string()));

let request = CompletionRequest {
    model: "claude-sonnet-4-20250514".to_string(),
    messages: vec![
        Message {
            role: Role::User,
            content: vec![ContentBlock::Text { text: "Hello!".to_string() }],
        }
    ],
    ..Default::default()
};

let response = provider.complete(request).await?;
```

---

## openclaw-agents

Agent runtime with sandboxing and workflow support.

### Modules

| Module | Description |
|--------|-------------|
| `runtime` | Agent execution environment |
| `sandbox` | Platform-specific isolation |
| `tools` | Tool registry and execution |
| `workflow` | Node graph workflows |

### Sandbox Configuration

```rust
pub struct SandboxConfig {
    pub level: SandboxLevel,
    pub allowed_paths: Vec<PathBuf>,
    pub network_access: bool,
    pub max_memory_mb: Option<u64>,
    pub timeout_seconds: Option<u64>,
}

pub enum SandboxLevel {
    None,      // No sandboxing
    Relaxed,   // Workspace access, network allowed
    Strict,    // Workspace only, no network
}
```

### Tool Definition

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Value;
    async fn execute(&self, params: Value, context: &ToolContext) -> Result<ToolResult, ToolError>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}
```

### Workflow Nodes

```rust
#[async_trait]
pub trait WorkflowNode: Send + Sync {
    fn id(&self) -> &str;
    fn node_type(&self) -> &str;
    async fn execute(&self, context: &mut NodeContext) -> Result<NodeOutput, WorkflowError>;
}

// Built-in nodes
pub struct InputNode { ... }     // Receive input
pub struct OutputNode { ... }    // Produce output
pub struct BranchNode { ... }    // Conditional branching
pub struct ToolNode { ... }      // Execute tool
pub struct LlmNode { ... }       // Call LLM
```

---

## openclaw-channels

Channel adapters for messaging platforms.

### Modules

| Module | Description |
|--------|-------------|
| `traits` | Channel trait definitions |
| `routing` | Message routing rules |
| `allowlist` | Access control |
| `registry` | Channel management |
| `telegram` | Telegram Bot API adapter |

### Channel Traits

```rust
#[async_trait]
pub trait Channel: Send + Sync {
    fn id(&self) -> &ChannelId;
    fn name(&self) -> &str;
    async fn start(&self) -> Result<(), ChannelError>;
    async fn stop(&self) -> Result<(), ChannelError>;
    async fn status(&self) -> ChannelStatus;
}

#[async_trait]
pub trait ChannelInbound: Channel {
    async fn receive(&self) -> Result<InboundMessage, ChannelError>;
}

#[async_trait]
pub trait ChannelOutbound: Channel {
    async fn send(&self, message: OutboundMessage) -> Result<MessageId, ChannelError>;
}
```

### Routing Rules

```rust
pub struct RouteRule {
    pub channel: Option<ChannelId>,
    pub peer_pattern: Option<String>,
    pub agent_id: AgentId,
    pub priority: i32,
}

pub struct AgentRouter {
    rules: Vec<RouteRule>,
    default_agent: AgentId,
}
```

---

## openclaw-gateway

HTTP/WebSocket server with JSON-RPC API.

### Modules

| Module | Description |
|--------|-------------|
| `server` | axum HTTP server with GatewayBuilder |
| `rpc` | JSON-RPC 2.0 handling |
| `middleware` | Auth, rate limiting |

### Server Configuration

```rust
pub struct GatewayConfig {
    pub bind_address: String,
    pub port: u16,
    pub tls: Option<TlsConfig>,
    pub cors_origins: Vec<String>,
}

// Builder pattern
let gateway = GatewayBuilder::new(config)
    .with_event_store(store)
    .with_agent("default", runtime)
    .with_tool_registry(tools)
    .build()?;
gateway.run().await?;
```

### RPC Methods

All methods are wired to the agent runtime and event store:

```rust
"session.create"    // Create session, log SessionStarted event
"session.message"   // Process via AgentRuntime, log events
"session.history"   // Query events from EventStore
"session.end"       // Log SessionEnded event

"agent.list"        // List registered agents
"agent.status"      // Get agent status

"tools.list"        // List registered tools
"tools.execute"     // Execute tool via ToolRegistry
```

---

## openclaw-plugins

Plugin system for extensibility.

### Modules

| Module | Description |
|--------|-------------|
| `api` | Plugin trait definitions |
| `registry` | Plugin management |
| `bridge` | TypeScript IPC bridge with process lifecycle |

### Plugin API

```rust
#[async_trait]
pub trait Plugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn hooks(&self) -> &[PluginHook];
    async fn execute_hook(&self, hook: PluginHook, data: Value) -> Result<Value, PluginError>;
    async fn activate(&self) -> Result<(), PluginError>;
    async fn deactivate(&self) -> Result<(), PluginError>;
}

pub enum PluginHook {
    BeforeMessage, AfterMessage,
    BeforeToolCall, AfterToolCall,
    SessionStart, SessionEnd,
    AgentResponse, Error,
}
```

### TypeScript Bridge

```rust
let mut bridge = TsPluginBridge::new(Path::new("./plugins"));
bridge.spawn_and_connect()?;   // Start plugin host process
let manifest = bridge.skill_manifest();  // Get available skills
bridge.execute_tool("search", json!({}))?;
bridge.stop();
```

### Plugin Discovery

```rust
let plugins = discover_plugins(Path::new("./plugins"));
// Scans for directories containing package.json with "openclaw" marker
```

---

## openclaw-cli

Command-line interface with full onboarding and management support.

### Commands

```
openclaw
├── onboard          # Interactive setup wizard
│   ├── --non-interactive
│   ├── --accept-risk
│   ├── --flow [quickstart|advanced]
│   └── --install-daemon
├── configure        # Update configuration
│   └── --section [auth|gateway|agents|channels|workspace]
├── doctor           # Health checks with auto-repair
│   ├── --repair
│   ├── --force
│   └── --deep
├── status           # Show system status
│   ├── --all
│   └── --deep
├── gateway
│   ├── run          # Start gateway server
│   │   ├── --port
│   │   ├── --bind
│   │   └── --force
│   └── status       # Check gateway status
├── channels
│   ├── --list       # List configured channels
│   └── --probe      # Check channel status
├── config
│   ├── get <key>    # Get config value
│   ├── set <key> <value>  # Set config value
│   ├── show         # Show full config
│   └── validate     # Validate config
├── completion       # Shell completion
│   ├── --shell [zsh|bash|fish|powershell]
│   ├── --install
│   └── --write-state
├── daemon           # System service management
│   ├── install
│   ├── uninstall
│   ├── start
│   ├── stop
│   └── status
└── reset            # Reset configuration
    ├── --config-only
    └── --all
```

### Onboarding Flow

```bash
# First run - interactive wizard
openclaw onboard

# Non-interactive for CI/automation
openclaw onboard \
  --non-interactive \
  --accept-risk \
  --flow quickstart \
  --auth-choice anthropic \
  --api-key "sk-..."
```

### Usage

```bash
# Check system status
openclaw status --all

# Start gateway
openclaw gateway run

# Run health checks with auto-repair
openclaw doctor --repair

# Configure authentication
openclaw configure --section auth

# Install shell completions
openclaw completion --install

# Install as system service
openclaw daemon install
openclaw daemon start
```

---

## openclaw-node

napi-rs bindings exposing Rust core functionality to Node.js.

### Modules

| Module | Description |
|--------|-------------|
| `config` | Configuration loading and validation |
| `providers` | Anthropic Claude and OpenAI GPT clients |
| `auth` | Safe API key handling and encrypted storage |
| `agents` | Tool registry and definitions |
| `events` | Append-only event store with projections |
| `validation` | Input validation utilities |

### AI Providers

```typescript
// Anthropic Claude
export class AnthropicProvider {
  constructor(apiKey: string);
  static withBaseUrl(apiKey: string, baseUrl: string): AnthropicProvider;
  get name(): string;
  listModels(): Promise<string[]>;
  complete(request: JsCompletionRequest): Promise<JsCompletionResponse>;
  completeStream(request: JsCompletionRequest, callback: StreamCallback): void;
}

// OpenAI GPT
export class OpenAIProvider {
  constructor(apiKey: string);
  static withBaseUrl(apiKey: string, baseUrl: string): OpenAIProvider;
  static withOrg(apiKey: string, orgId: string): OpenAIProvider;
  get name(): string;
  listModels(): Promise<string[]>;
  complete(request: JsCompletionRequest): Promise<JsCompletionResponse>;
  completeStream(request: JsCompletionRequest, callback: StreamCallback): void;
}
```

### Provider Types

```typescript
interface JsMessage {
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string;
  toolUseId?: string;
  toolName?: string;
}

interface JsCompletionRequest {
  model: string;
  messages: JsMessage[];
  system?: string;
  maxTokens: number;
  temperature?: number;
  stop?: string[];
  tools?: JsTool[];
}

interface JsCompletionResponse {
  id: string;
  model: string;
  content: string;
  stopReason?: 'end_turn' | 'max_tokens' | 'stop_sequence' | 'tool_use';
  toolCalls?: JsToolCall[];
  usage: JsTokenUsage;
}

interface JsStreamChunk {
  chunkType: string;
  delta?: string;
  index?: number;
  stopReason?: string;
}

interface JsTokenUsage {
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens?: number;
  cacheWriteTokens?: number;
}
```

### Authentication

```typescript
// Safe API key wrapper (prevents accidental logging)
export class NodeApiKey {
  constructor(key: string);
  toString(): string;  // Returns "[REDACTED]"
  exposeSecretForApiCall(): string;  // Get actual value for API calls
  isEmpty(): boolean;
  length(): number;
  startsWith(prefix: string): boolean;
}

// Encrypted credential storage (AES-256-GCM)
export class CredentialStore {
  constructor(encryptionKeyHex: string, storePath: string);
  store(name: string, apiKey: NodeApiKey): Promise<void>;
  load(name: string): Promise<NodeApiKey>;
  delete(name: string): Promise<void>;
  list(): Promise<string[]>;
}
```

### Tools

```typescript
export class ToolRegistry {
  constructor();
  register(tool: JsToolDefinition): void;
  list(): string[];
  execute(name: string, params: object): Promise<JsToolResult>;
}

interface JsToolDefinition {
  name: string;
  description: string;
  inputSchema: object;
}

interface JsToolResult {
  success: boolean;
  result?: any;
  error?: string;
}
```

### Configuration

```typescript
export function loadConfig(path: string): string;
export function loadDefaultConfig(): string;
export function validateConfig(path: string): string;
```

### Session Key

```typescript
export function buildSessionKey(
  agentId: string, channel: string, accountId: string,
  peerType: string, peerId: string
): string;
```

### Validation

```typescript
export function validateMessage(content: string, maxLength?: number): string;
export function validatePath(path: string): string;
```

### Event Store

```typescript
export class NodeEventStore {
  constructor(path: string);
  appendEvent(sessionKey: string, agentId: string, eventType: string, data: string): string;
  getEvents(sessionKey: string): string;
  getProjection(sessionKey: string): string;
  listSessions(): string[];
  flush(): void;
}
```

### Supported Event Types

- `session_started` - Session opened (channel, peer_id)
- `message_received` - Incoming message (content)
- `message_sent` - Outgoing message (content, message_id)
- `agent_response` - Agent reply (content, model, tokens)
- `session_ended` - Session closed (reason)
- `state_changed` - State mutation (key, value)
- `tool_called` - Tool invocation (tool_name, params)
- `tool_result` - Tool result (tool_name, result, success)

### Usage Example

```javascript
const {
  AnthropicProvider,
  NodeApiKey,
  CredentialStore,
  NodeEventStore,
} = require('openclaw-node');

// Create provider
const provider = new AnthropicProvider(process.env.ANTHROPIC_API_KEY);

// Create completion
const response = await provider.complete({
  model: 'claude-3-5-sonnet-20241022',
  messages: [{ role: 'user', content: 'Hello!' }],
  maxTokens: 1024,
});

console.log(response.content);

// Streaming
provider.completeStream(request, (err, chunk) => {
  if (err) console.error(err);
  else if (chunk.delta) process.stdout.write(chunk.delta);
});
```
