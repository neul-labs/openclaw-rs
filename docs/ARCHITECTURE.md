# Architecture

OpenClaw Rust Core follows a modular, security-first architecture designed for high performance and TypeScript interoperability.

## High-Level Overview

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                                    Clients                                       │
├─────────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────────────────┐  │
│  │  macOS App  │  │  iOS App    │  │ Android App │  │   TS Plugins (IPC)    │  │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └───────────┬───────────┘  │
│         │                │                │                      │              │
│         └────────────────┴────────────────┴──────────────────────┘              │
│                                     │                                            │
│                          HTTP/WS • JSON-RPC 2.0                                  │
│                                     │                                            │
├─────────────────────────────────────┴───────────────────────────────────────────┤
│                                   Gateway                                        │
│  ┌──────────────────────────────────────────────────────────────────────────┐   │
│  │                          openclaw-gateway                                 │   │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────────────┐  │   │
│  │  │ axum HTTP  │  │ WebSocket  │  │  JSON-RPC  │  │   Rate Limiting    │  │   │
│  │  │   Server   │  │  Handler   │  │  Dispatch  │  │    Middleware      │  │   │
│  │  └────────────┘  └────────────┘  └────────────┘  └────────────────────┘  │   │
│  └──────────────────────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────────────────┤
│                               Core Services                                      │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────────────────┐  │
│  │  openclaw-agents │  │openclaw-channels │  │     openclaw-plugins         │  │
│  │  ┌────────────┐  │  │  ┌────────────┐  │  │  ┌────────────────────────┐  │  │
│  │  │  Runtime   │  │  │  │  Telegram  │  │  │  │    Plugin Registry     │  │  │
│  │  │  Sandbox   │  │  │  │  Discord   │  │  │  │    TypeScript Bridge   │  │  │
│  │  │  Workflow  │  │  │  │  Slack     │  │  │  │    WASM Runtime        │  │  │
│  │  │  Tools     │  │  │  │  Signal    │  │  │  └────────────────────────┘  │  │
│  │  └────────────┘  │  │  │  Matrix    │  │  └──────────────────────────────┘  │
│  └──────────────────┘  │  │  WhatsApp  │  │                                     │
│                        │  └────────────┘  │                                     │
│                        └──────────────────┘                                     │
├─────────────────────────────────────────────────────────────────────────────────┤
│                               Foundation                                         │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────────────────┐  │
│  │   openclaw-core  │  │openclaw-providers│  │       openclaw-ipc           │  │
│  │  ┌────────────┐  │  │  ┌────────────┐  │  │  ┌────────────────────────┐  │  │
│  │  │   Types    │  │  │  │ Anthropic  │  │  │  │    nng Transport       │  │  │
│  │  │   Config   │  │  │  │  OpenAI    │  │  │  │    Message Types       │  │  │
│  │  │   Events   │  │  │  │  Google    │  │  │  │    Request/Response    │  │  │
│  │  │   Secrets  │  │  │  │  Ollama    │  │  │  └────────────────────────┘  │  │
│  │  │   Auth     │  │  │  └────────────┘  │  └──────────────────────────────┘  │
│  │  └────────────┘  │  └──────────────────┘                                     │
│  └──────────────────┘                                                           │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## Design Principles

| Principle | Description | Implementation |
|-----------|-------------|----------------|
| **Event Sourcing** | All state changes are stored as immutable events | `EventStore` with sled, append-only logs |
| **CRDT Projections** | Derived state supports conflict-free merging | `SessionProjection` with LWW semantics |
| **Defense in Depth** | Multiple layers of security controls | Validation → Sandboxing → Audit |
| **Zero Trust** | All external input is untrusted | Boundary validation in every crate |
| **Fail Secure** | Errors default to denial | `Result<T, E>` everywhere, no panics |
| **Least Privilege** | Minimal permissions per component | Sandbox policies, capability-based |

## Crate Dependency Graph

```
                              ┌─────────────────┐
                              │  openclaw-cli   │
                              └────────┬────────┘
                                       │
                    ┌──────────────────┼──────────────────┐
                    │                  │                  │
                    ▼                  ▼                  ▼
          ┌─────────────────┐ ┌───────────────┐ ┌─────────────────┐
          │openclaw-gateway │ │openclaw-agents│ │ openclaw-core   │
          └────────┬────────┘ └───────┬───────┘ └────────┬────────┘
                   │                  │                  │
         ┌─────────┼─────────┐        │                  │
         │         │         │        │                  │
         ▼         ▼         ▼        ▼                  │
┌─────────────┐ ┌─────────┐ ┌──────────────┐             │
│  openclaw-  │ │openclaw-│ │  openclaw-   │             │
│  channels   │ │ plugins │ │  providers   │             │
└──────┬──────┘ └────┬────┘ └──────┬───────┘             │
       │             │             │                     │
       └─────────────┴─────────────┴─────────────────────┘
                              │
                              ▼
                     ┌─────────────────┐
                     │   openclaw-ipc  │
                     └─────────────────┘
```

## Data Flow

### Message Processing

```
┌──────────┐     ┌───────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│  Channel │────▶│  Gateway  │────▶│  Router  │────▶│  Agent   │────▶│ Provider │
│ Adapter  │     │           │     │          │     │ Runtime  │     │          │
└──────────┘     └───────────┘     └──────────┘     └──────────┘     └──────────┘
     │                                                    │
     │                                                    │
     │           ┌───────────┐                           │
     └───────────│  Event    │◀──────────────────────────┘
                 │  Store    │
                 └───────────┘
```

1. **Inbound**: Channel adapter receives message → validates → routes to agent
2. **Processing**: Agent runtime executes workflow → calls tools → queries provider
3. **Event Capture**: All state changes recorded in event store
4. **Outbound**: Response flows back through gateway to channel

### Session Lifecycle

```
SessionStarted ──▶ MessageReceived ──▶ ToolCalled ──▶ ToolResult ──▶ AgentResponse ──▶ MessageSent
                        │                                                                  │
                        └──────────────────────────────────────────────────────────────────┘
                                              (repeat)
                                                 │
                                                 ▼
                                          SessionEnded
```

## Security Boundaries

```
┌─────────────────────────────────────────────────────────────────┐
│                         UNTRUSTED                                │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  External Messages (Telegram, Discord, etc.)              │  │
│  │  User Input                                                │  │
│  │  Plugin Code                                               │  │
│  └───────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                     VALIDATION BOUNDARY                          │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  validate_message_content()                               │  │
│  │  Allowlist checks                                          │  │
│  │  Rate limiting                                             │  │
│  └───────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                     SANDBOX BOUNDARY                             │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Tool execution (bwrap/sandbox-exec)                      │  │
│  │  Plugin isolation                                          │  │
│  │  Network restrictions                                      │  │
│  └───────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                         TRUSTED                                  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Core types and logic                                      │  │
│  │  Event store                                               │  │
│  │  Configuration                                             │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Pattern References

### Grite Pattern (Event Sourcing)

From [grite](https://github.com/anthropics/grite):

- **Append-only event logs**: Sessions stored as immutable event sequences
- **CRDT projections**: Materialized views with conflict-free merge
- **Content-addressed IDs**: Events identified by BLAKE2b hash
- **sled storage**: Fast embedded database with ACID guarantees

### M9M Pattern (Workflow Nodes)

From [m9m](https://github.com/anthropics/m9m):

- **Node graph execution**: Workflows as directed graphs of nodes
- **Platform sandboxing**: OS-specific isolation (bwrap, sandbox-exec, Job Objects)
- **Tool registry**: Dynamic tool discovery and execution
- **Capability-based security**: Nodes request specific permissions

## Storage Layout

```
~/.openclaw/
├── openclaw.json           # Configuration (JSON5)
├── credentials/            # Encrypted API keys
│   ├── anthropic.enc
│   ├── openai.enc
│   └── ...
├── sessions/               # Event store (sled)
│   ├── events/
│   └── projections/
└── agents/
    └── <agent-id>/
        ├── sessions/       # Agent-specific sessions
        └── state/          # Agent state
```

## Thread Model

```
┌─────────────────────────────────────────────────────────────────┐
│                        Tokio Runtime                             │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ HTTP Server │  │  WS Handler │  │   Background Tasks      │  │
│  │   (spawn)   │  │   (spawn)   │  │  (session cleanup, etc) │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
│         │                │                      │                │
│         └────────────────┴──────────────────────┘                │
│                          │                                       │
│                   ┌──────┴──────┐                               │
│                   │   Shared    │                               │
│                   │   State     │                               │
│                   │  (Arc<RwLock>)                              │
│                   └─────────────┘                               │
└─────────────────────────────────────────────────────────────────┘
```

- **Async-first**: All I/O operations are async via tokio
- **Shared state**: Protected by `Arc<RwLock<T>>` or `Arc<Mutex<T>>`
- **No unsafe**: All crates use `#![forbid(unsafe_code)]`
- **Backpressure**: Bounded channels prevent memory exhaustion
