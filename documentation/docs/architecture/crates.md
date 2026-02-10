# Crate Structure

!!! info "Community Implementation"
    This is a community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw).

openclaw-rs is organized as a Cargo workspace with multiple crates.

---

## Workspace Overview

```
openclaw-rs/
├── crates/
│   ├── openclaw-core/         # Core types and auth
│   ├── openclaw-providers/    # AI provider implementations
│   ├── openclaw-agents/       # Agent runtime
│   ├── openclaw-channels/     # Pub/sub messaging
│   ├── openclaw-plugins/      # Plugin system
│   ├── openclaw-ipc/          # IPC bridge
│   ├── openclaw-gateway/      # HTTP/WS server
│   ├── openclaw-cli/          # Command-line tool
│   └── openclaw-ui/           # Vue 3 web UI
└── bridge/
    └── openclaw-node/         # Node.js bindings
```

---

## Dependency Graph

```
                    ┌─────────────┐
                    │ openclaw-cli│
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │  openclaw-  │
                    │   gateway   │
                    └──────┬──────┘
                           │
       ┌───────────────────┼───────────────────┐
       │                   │                   │
┌──────▼──────┐     ┌──────▼──────┐     ┌──────▼──────┐
│  openclaw-  │     │  openclaw-  │     │  openclaw-  │
│   agents    │     │  channels   │     │   plugins   │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
                    ┌──────▼──────┐
                    │  openclaw-  │
                    │  providers  │
                    └──────┬──────┘
                           │
       ┌───────────────────┴───────────────────┐
       │                                       │
┌──────▼──────┐                         ┌──────▼──────┐
│  openclaw-  │                         │  openclaw-  │
│    core     │                         │     ipc     │
└─────────────┘                         └─────────────┘
```

---

## Crate Details

### openclaw-core

**Foundation types and authentication.**

```toml
[dependencies]
openclaw-core = "0.1"
```

| Feature | Description |
|---------|-------------|
| `types` | Common type definitions |
| `auth` | Credential management |
| `config` | Configuration loading |
| `events` | Event system primitives |

Key types:

- `Message` - Chat message structure
- `Tool` - Tool definition
- `CredentialStore` - Encrypted credential storage
- `Config` - Application configuration

---

### openclaw-providers

**AI provider implementations.**

```toml
[dependencies]
openclaw-providers = "0.1"
```

| Provider | Status | Features |
|----------|--------|----------|
| Anthropic | Full | Streaming, tools |
| OpenAI | Full | Streaming, functions |

Key traits:

- `Provider` - Common provider interface
- `StreamingProvider` - Streaming support
- `ToolProvider` - Tool calling support

---

### openclaw-agents

**Agent runtime and orchestration.**

```toml
[dependencies]
openclaw-agents = "0.1"
```

| Feature | Description |
|---------|-------------|
| `runtime` | Agent execution |
| `tools` | Tool management |
| `sessions` | Session tracking |

Key types:

- `AgentRuntime` - Main runtime
- `Session` - Conversation session
- `ToolExecutor` - Tool execution

---

### openclaw-channels

**Pub/sub messaging system.**

```toml
[dependencies]
openclaw-channels = "0.1"
```

Provides:

- Topic-based messaging
- Broadcast channels
- Event filtering
- Async subscribers

---

### openclaw-plugins

**Plugin system with IPC support.**

```toml
[dependencies]
openclaw-plugins = "0.1"
```

Enables:

- External process plugins
- JSON-RPC communication
- Capability extension
- Sandboxed execution

---

### openclaw-ipc

**Inter-process communication.**

```toml
[dependencies]
openclaw-ipc = "0.1"
```

Features:

- JSON-RPC client/server
- Stdio transport
- Socket transport
- Process lifecycle

---

### openclaw-gateway

**HTTP/WebSocket server.**

```toml
[dependencies]
openclaw-gateway = "0.1"
```

Includes:

- Axum-based HTTP server
- WebSocket support
- Embedded UI serving
- REST and JSON-RPC APIs

---

### openclaw-cli

**Command-line interface.**

```bash
cargo install openclaw-cli
```

Commands:

- `onboard` - Setup wizard
- `gateway` - Server management
- `config` - Configuration
- `status` - System status

---

### openclaw-ui

**Vue 3 web dashboard.**

Not published to crates.io - embedded in gateway.

Built with:

- Vue 3 + Composition API
- Pinia state management
- Vue Router
- Vite build tool

---

### openclaw-node

**Node.js bindings.**

```bash
npm install openclaw-node
```

Provides:

- Native provider bindings
- Event system
- Auth integration
- Agent runtime access

---

## Feature Flags

### openclaw-core

| Flag | Description |
|------|-------------|
| `default` | All features |
| `auth` | Credential storage |
| `config` | Configuration loading |

### openclaw-providers

| Flag | Description |
|------|-------------|
| `default` | All providers |
| `anthropic` | Anthropic only |
| `openai` | OpenAI only |

### openclaw-gateway

| Flag | Description |
|------|-------------|
| `default` | Full gateway |
| `embedded-ui` | Include web UI |

---

## Publishing Order

For crates.io publishing, use this order:

1. `openclaw-core`
2. `openclaw-ipc`
3. `openclaw-providers`
4. `openclaw-channels`
5. `openclaw-plugins`
6. `openclaw-agents`
7. `openclaw-gateway`
8. `openclaw-cli`
9. `openclaw-node` (npm)

---

## Next Steps

[:material-lightbulb: Design Principles](design.md){ .md-button .md-button--primary }
[:material-shield: Security Model](security.md){ .md-button }
