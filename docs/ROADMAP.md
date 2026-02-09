# Roadmap

This document tracks the implementation progress of OpenClaw Rust Core and outlines the migration path from TypeScript.

## Migration Strategy

The Rust core is being built as a **gradual replacement**, not a rewrite:

1. Build Rust components in parallel with TypeScript
2. Expose Rust functionality via napi-rs bindings
3. Replace TypeScript modules one-by-one
4. Maintain full backward compatibility throughout

## Implementation Phases

### Phase 1: Core Library ✅

**Status**: Complete

Foundation types, configuration, event sourcing, and security primitives.

| Component | Status | Notes |
|-----------|--------|-------|
| Types (`AgentId`, `SessionKey`, etc.) | ✅ Done | Full type system |
| Configuration (JSON5) | ✅ Done | Compatible with TS format |
| Event Store (sled) | ✅ Done | Append-only, CRDT projections |
| Secrets Management | ✅ Done | AES-256-GCM encryption |
| Input Validation | ✅ Done | Size limits, sanitization |
| Authentication | ✅ Done | OAuth token management |

### Phase 2: IPC & Providers ✅

**Status**: Complete

Inter-process communication and AI provider clients.

| Component | Status | Notes |
|-----------|--------|-------|
| IPC Message Types | ✅ Done | Request/Response/Event |
| nng Transport | ⚠️ Stub | Needs nng integration |
| Provider Traits | ✅ Done | `Provider`, `CompletionRequest` |
| Anthropic Client | ✅ Done | Full API client with SSE streaming |
| OpenAI Client | ✅ Done | Full API client with SSE streaming |
| Usage Tracking | ✅ Done | Token counting with cache fields |

### Phase 3: Agent Runtime ✅

**Status**: Complete (structure), Partial (execution)

Agent execution environment with sandboxing and workflow support.

| Component | Status | Notes |
|-----------|--------|-------|
| Runtime Core | ✅ Done | `AgentRuntime`, context |
| Sandbox (Linux) | ✅ Done | bubblewrap integration |
| Sandbox (macOS) | ✅ Done | sandbox-exec profiles |
| Sandbox (Windows) | ⚠️ Stub | Job Objects planned |
| Tool Registry | ✅ Done | Dynamic registration |
| Bash Tool | ✅ Done | Command execution |
| Workflow Engine | ✅ Done | Node graph execution |
| Workflow Nodes | ✅ Done | Input, Output, Branch |

### Phase 4: Channels ⚠️

**Status**: In Progress

Channel adapters for messaging platforms.

| Component | Status | Notes |
|-----------|--------|-------|
| Channel Traits | ✅ Done | Inbound/Outbound |
| Routing | ✅ Done | Rule-based routing |
| Allowlist | ✅ Done | Access control |
| Registry | ✅ Done | Channel management |
| Telegram Adapter | ✅ Done | Full Bot API with attachments |
| Discord Adapter | 🔜 Planned | |
| Slack Adapter | 🔜 Planned | |
| Signal Adapter | 🔜 Planned | |
| Matrix Adapter | 🔜 Planned | |
| WhatsApp Adapter | 🔜 Planned | |

### Phase 5: Gateway ✅

**Status**: Complete

HTTP/WebSocket server with JSON-RPC API.

| Component | Status | Notes |
|-----------|--------|-------|
| HTTP Server (axum) | ✅ Done | Basic routes |
| WebSocket Handler | ⚠️ Stub | Connection management |
| JSON-RPC 2.0 | ✅ Done | Request/Response types |
| RPC Methods | ✅ Done | 8 methods wired to agents/events |
| Rate Limiting | ✅ Done | Per-client limits |
| Middleware | ✅ Done | Auth, logging |
| GatewayBuilder | ✅ Done | Fluent builder API |

### Phase 6: Plugins ⚠️

**Status**: In Progress

Plugin system for extensibility.

| Component | Status | Notes |
|-----------|--------|-------|
| Plugin API | ✅ Done | Traits, hooks |
| Plugin Registry | ✅ Done | Registration, lookup |
| TypeScript Bridge | ✅ Done | IPC bridge with process lifecycle |
| Plugin Discovery | ✅ Done | Scans for package.json markers |
| WASM Runtime | 🔜 Planned | Sandboxed plugins |
| Native Plugins | 🔜 Planned | FFI interface |

### Phase 7: CLI & Node Bindings ⚠️

**Status**: In Progress

Command-line interface and Node.js integration.

| Component | Status | Notes |
|-----------|--------|-------|
| CLI Framework | ✅ Done | clap with subcommands |
| `onboard` | ✅ Done | Interactive setup wizard |
| `configure` | ✅ Done | Configuration updates |
| `doctor` | ✅ Done | Health checks with auto-repair |
| `status` | ✅ Done | Gateway/channels status |
| `gateway run` | ✅ Done | Start server |
| `completion` | ✅ Done | Shell completion setup |
| `daemon` | ✅ Done | System service management |
| `config get/set` | ✅ Done | Configuration management |
| `reset` | ✅ Done | Reset configuration/state |
| napi-rs Bindings | ✅ Done | Config, events, validation, session keys |

## Progress Summary

| Crate | Status | Completion |
|-------|--------|------------|
| `openclaw-core` | ✅ Complete | 100% |
| `openclaw-ipc` | ✅ Complete | 90% |
| `openclaw-providers` | ✅ Complete | 85% |
| `openclaw-agents` | ✅ Complete | 85% |
| `openclaw-channels` | ⚠️ Partial | 50% |
| `openclaw-gateway` | ✅ Complete | 80% |
| `openclaw-plugins` | ⚠️ Partial | 70% |
| `openclaw-cli` | ✅ Complete | 90% |
| `openclaw-node` | ✅ Complete | 80% |

## Performance Targets

| Metric | Target | Current |
|--------|--------|---------|
| Cold start | < 50ms | TBD |
| Message latency | < 10ms | TBD |
| Memory (idle) | < 20MB | TBD |
| Binary size | < 15MB | TBD |

## Compatibility Guarantees

Throughout the migration:

- ✅ Same configuration format (`~/.openclaw/openclaw.json`)
- ✅ Same session storage location (`~/.openclaw/sessions/`)
- ✅ TypeScript plugins continue to work via IPC
- ✅ Skills format unchanged (Markdown + YAML frontmatter)
- ✅ CLI commands maintain same interface

## Open Questions

1. **WASM Plugin Runtime**: Use wasmtime or wasmer?
2. **Mobile Builds**: Priority of iOS/Android support?
3. **Observability**: OpenTelemetry integration scope?
4. **Clustering**: Multi-instance coordination strategy?

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to help with implementation.

Priority areas:
1. Channel adapters (Discord, Slack, Signal)
2. WebSocket handler for gateway
3. WASM plugin runtime
4. Documentation and examples
