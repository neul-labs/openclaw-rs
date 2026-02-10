# Architecture Overview

!!! info "Community Implementation"
    A Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw) by [Neul Labs](https://neullabs.com).

This section describes the high-level architecture of openclaw-rs.

---

## System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         Applications                             │
├─────────────┬─────────────────┬─────────────────────────────────┤
│  CLI Tool   │  Web Dashboard  │     Node.js Applications        │
│(openclaw-cli)│ (openclaw-ui)  │      (openclaw-node)            │
└──────┬──────┴────────┬────────┴───────────────┬─────────────────┘
       │               │                        │
       ▼               ▼                        ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Gateway Server                               │
│                    (openclaw-gateway)                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │  REST API   │  │  WebSocket  │  │  Embedded UI (Vue 3)   │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└──────────────────────────┬──────────────────────────────────────┘
                           │
       ┌───────────────────┼───────────────────┐
       ▼                   ▼                   ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Agents    │     │  Channels   │     │   Plugins   │
│(openclaw-   │     │ (openclaw-  │     │ (openclaw-  │
│  agents)    │     │  channels)  │     │  plugins)   │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │                   │                   │
       ▼                   ▼                   ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Core Libraries                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │  Providers  │  │    Auth     │  │         IPC             │  │
│  │ (openclaw-  │  │ (openclaw-  │  │    (openclaw-ipc)       │  │
│  │  providers) │  │   core)     │  │                         │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                     AI Provider APIs                             │
│              (Anthropic, OpenAI, etc.)                           │
└─────────────────────────────────────────────────────────────────┘
```

---

## Core Components

### Gateway (openclaw-gateway)

The central HTTP/WebSocket server that:

- Serves the web dashboard
- Exposes REST and JSON-RPC APIs
- Manages WebSocket connections
- Coordinates agent sessions
- Routes tool execution

### CLI (openclaw-cli)

Command-line interface for:

- Gateway management
- Configuration
- Session viewing
- System diagnostics

### Providers (openclaw-providers)

AI provider implementations:

- Anthropic (Claude models)
- OpenAI (GPT models)
- Unified trait interface
- Streaming support

### Agents (openclaw-agents)

Agent runtime system:

- Message handling
- Tool orchestration
- Session management
- Context tracking

### Channels (openclaw-channels)

Pub/sub messaging:

- Event distribution
- Component coordination
- State synchronization

### Plugins (openclaw-plugins)

Plugin system:

- IPC bridge to external processes
- Dynamic loading
- Capability extension

---

## Data Flow

### Request Flow

```
1. User sends message via CLI/Dashboard/API
2. Gateway receives and routes to agent
3. Agent constructs request with context
4. Provider sends to AI API
5. Response streams back through agent
6. Gateway distributes to connected clients
```

### Tool Execution Flow

```
1. AI requests tool use
2. Agent validates request
3. Gateway routes to appropriate handler
4. Tool executes in sandbox
5. Result returns to agent
6. Agent feeds result to AI
7. AI continues generation
```

---

## Technology Stack

| Layer | Technology |
|-------|------------|
| Language | Rust 2024 edition |
| Async Runtime | Tokio |
| HTTP Server | Axum |
| WebSocket | tokio-tungstenite |
| Serialization | serde |
| Node Bindings | napi-rs |
| Web UI | Vue 3, Pinia, Vite |

---

## Key Principles

### 1. Type Safety

Rust's type system ensures:

- API contracts at compile time
- No null pointer exceptions
- Thread-safe by default

### 2. Async-First

Built on Tokio for:

- High concurrency
- Efficient I/O
- Streaming responses

### 3. Modularity

Separate crates for:

- Clean dependencies
- Independent versioning
- Optional features

### 4. Compatibility

Designed for interoperability with:

- Official OpenClaw project
- Existing configurations
- Multiple AI providers

---

## Next Steps

[:material-package-variant: Crate Structure](crates.md){ .md-button .md-button--primary }
[:material-lightbulb: Design Principles](design.md){ .md-button }
