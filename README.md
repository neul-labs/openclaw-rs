# OpenClaw Rust Core

> **A Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw) by [Neul Labs](https://neullabs.com)**

[![Crates.io](https://img.shields.io/crates/v/openclaw-core.svg)](https://crates.io/crates/openclaw-core)
[![Documentation](https://docs.rs/openclaw-core/badge.svg)](https://docs.rs/openclaw-core)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

---

## What is This?

This is a **Rust implementation** of [OpenClaw](https://github.com/openclaw/openclaw), the open-source AI agent framework, developed by [Neul Labs](https://neullabs.com).

### Why Rust?

| Benefit | Impact |
|---------|--------|
| **Performance** | Sub-millisecond message routing, minimal memory footprint |
| **Safety** | Memory safety without GC, thread safety via ownership |
| **Reliability** | No null pointer exceptions, exhaustive pattern matching |
| **Portability** | Single binary deployment, cross-compilation to mobile/embedded |
| **Interop** | napi-rs bindings for Node.js compatibility |

---

## Project Structure

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              OpenClaw Rust Core                                  │
├─────────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────────────────┐  │
│  │  macOS App  │  │  iOS App    │  │ Android App │  │   TS Plugins (IPC)    │  │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └───────────┬───────────┘  │
│         └────────────────┴────────────────┴──────────────────────┘              │
│                              HTTP/WS • JSON-RPC                                  │
├─────────────────────────────────────────────────────────────────────────────────┤
│                       ┌──────────────────────────┐                              │
│                       │     openclaw-gateway     │                              │
│                       │   (axum HTTP/WS server)  │                              │
│                       │  ┌──────────────────┐    │                              │
│                       │  │   openclaw-ui    │    │                              │
│                       │  │ (Vue 3 Dashboard)│    │                              │
│                       │  └──────────────────┘    │                              │
│                       └────────────┬─────────────┘                              │
│            ┌───────────────────────┼───────────────────────┐                    │
│            │                       │                       │                    │
│   ┌────────┴────────┐   ┌─────────┴─────────┐   ┌────────┴────────┐            │
│   │  openclaw-core  │   │  openclaw-agents  │   │openclaw-channels│            │
│   │  (types, config │   │  (runtime, sandbox│   │ (Telegram, etc.)│            │
│   │   events, auth) │   │   workflow, tools)│   └─────────────────┘            │
│   └─────────────────┘   └───────────────────┘                                   │
│            │                       │                                            │
│   ┌────────┴────────┐   ┌─────────┴─────────┐                                  │
│   │ openclaw-secrets│   │openclaw-providers │                                  │
│   │  (AES-256-GCM)  │   │(Anthropic, OpenAI)│                                  │
│   └─────────────────┘   └───────────────────┘                                   │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## Crates

| Crate | Status | Description |
|-------|--------|-------------|
| [`openclaw-core`](crates/openclaw-core) | ✅ Complete | Types, config, events, secrets, auth, validation |
| [`openclaw-ipc`](crates/openclaw-ipc) | ✅ Complete | IPC message types, nng transport |
| [`openclaw-providers`](crates/openclaw-providers) | ✅ Complete | Anthropic, OpenAI providers with streaming |
| [`openclaw-agents`](crates/openclaw-agents) | ✅ Complete | Runtime, sandbox, workflow, tools |
| [`openclaw-channels`](crates/openclaw-channels) | ⚠️ Partial | Channel traits, routing, allowlist |
| [`openclaw-gateway`](crates/openclaw-gateway) | ✅ Complete | HTTP/WS server, JSON-RPC, embedded UI |
| [`openclaw-plugins`](crates/openclaw-plugins) | ⚠️ Partial | Plugin API, FFI bridge (wasmtime) |
| [`openclaw-cli`](crates/openclaw-cli) | ✅ Complete | CLI commands (onboard, gateway, status, config) |
| [`openclaw-node`](bridge/openclaw-node) | ✅ Complete | napi-rs bindings for Node.js |
| [`openclaw-ui`](crates/openclaw-ui) | ✅ Complete | Vue 3 web dashboard (embedded in gateway) |

---

## Quick Start

### Installation

```bash
# From crates.io
cargo install openclaw-cli

# From source
git clone https://github.com/neul-labs/openclaw-rs
cd openclaw-rs
cargo install --path crates/openclaw-cli
```

### Requirements

- Rust 1.85+ (2024 edition)
- Node.js 20+ (for building the embedded UI)
- System dependencies for sandboxing:
  - Linux: `bubblewrap` (`bwrap`)
  - macOS: Built-in `sandbox-exec`
  - Windows: No additional deps (uses Job Objects)

### First Run

```bash
# Interactive setup wizard
openclaw onboard

# Start the gateway server
openclaw gateway run

# Open the web dashboard
openclaw dashboard
```

---

## Usage Examples

### CLI

```bash
# Check system status
openclaw status

# Run health checks
openclaw doctor

# Configure providers
openclaw configure --section auth

# Manage agents
openclaw agents list
```

### Node.js (via openclaw-node)

```javascript
const { AnthropicProvider, NodeEventStore } = require('openclaw-node');

// Create provider
const provider = new AnthropicProvider(process.env.ANTHROPIC_API_KEY);

// Create completion
const response = await provider.complete({
  model: 'claude-3-5-sonnet-20241022',
  messages: [{ role: 'user', content: 'Hello!' }],
  maxTokens: 1024,
});

console.log(response.content);
```

### Rust

```rust
use openclaw_providers::{AnthropicProvider, Provider};
use openclaw_core::secrets::ApiKey;

let provider = AnthropicProvider::new(ApiKey::new("sk-...".into()));
let response = provider.complete(request).await?;
```

---

## Design Principles

| Principle | Implementation |
|-----------|----------------|
| **Event Sourcing** | Sessions stored as append-only event logs (sled) |
| **CRDT Projections** | Conflict-free state via last-write-wins merge |
| **Defense in Depth** | Input validation, sandboxing, secret redaction |
| **Zero Trust** | All external input validated at boundaries |
| **Fail Secure** | Errors default to denial, not exposure |

---

## Documentation

- [Full Documentation](https://docs.neullabs.com/openclaw-rs/) - Complete guides and API reference
- [Architecture](docs/ARCHITECTURE.md) - System design and patterns
- [Security](docs/SECURITY.md) - Threat model and security measures
- [Crates](docs/CRATES.md) - Detailed crate documentation
- [Contributing](docs/CONTRIBUTING.md) - Development guide

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines.

---

## License

MIT License - see [LICENSE](LICENSE) for details.

---

## Acknowledgments

This project is a Rust implementation inspired by and compatible with the original [OpenClaw](https://github.com/openclaw/openclaw) project. We're grateful for:

- The excellent design and architecture of the original OpenClaw project
- The open-source community that makes projects like this possible
- The Rust ecosystem for its emphasis on safety and performance

---

## Legal Notice

- "OpenClaw" refers to the original open-source project at [github.com/openclaw/openclaw](https://github.com/openclaw/openclaw)
- "Claude" and "Anthropic" are trademarks of Anthropic, PBC
- "GPT" and "OpenAI" are trademarks of OpenAI, Inc
- All trademarks belong to their respective owners

This is an independent implementation by Neul Labs. Provider integrations use official public APIs.

---

## Links

- [Original OpenClaw](https://github.com/openclaw/openclaw) - The original TypeScript project
- [Documentation](https://docs.neullabs.com/openclaw-rs/) - Full documentation
- [GitHub](https://github.com/neul-labs/openclaw-rs) - Source code
- [Neul Labs](https://neullabs.com) - Organization
