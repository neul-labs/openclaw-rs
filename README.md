# OpenClaw Rust Core

> **A community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw)**

[![Crates.io](https://img.shields.io/crates/v/openclaw-core.svg)](https://crates.io/crates/openclaw-core)
[![Documentation](https://docs.rs/openclaw-core/badge.svg)](https://docs.rs/openclaw-core)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

---

## What is This?

This is an **unofficial, community-driven Rust implementation** of [OpenClaw](https://github.com/openclaw/openclaw), the popular open-source AI agent framework.

**This is NOT the official OpenClaw project.** For the official project, visit:
- [OpenClaw (Official)](https://github.com/openclaw/openclaw) - The main TypeScript implementation
- [OpenClaw Discord](https://discord.gg/openclaw) - Official community chat
- [OpenClaw Docs](https://docs.openclaw.dev) - Official documentation

### Why This Exists

We love OpenClaw and wanted to explore what a Rust implementation might look like. This project is:

- A **tribute** to the excellent design of the original OpenClaw
- An **experiment** in bringing Rust's performance and safety guarantees to the AI agent space
- A **learning project** for understanding AI agent architectures
- **Fully compatible** with OpenClaw's config format, skills, and plugin ecosystem

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
git clone https://github.com/openclaw/openclaw-rs
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

## Compatibility with OpenClaw

This implementation aims for compatibility with the official OpenClaw project:

| Feature | Compatibility |
|---------|---------------|
| **Config Format** | ✅ Same `~/.openclaw/openclaw.json` (JSON5) |
| **Skills** | ✅ Markdown + YAML frontmatter format |
| **Plugins** | ✅ TypeScript plugins via IPC bridge |
| **Session Storage** | ✅ Compatible event format |

See [docs/INTEROP.md](docs/INTEROP.md) for the full compatibility guide.

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

- [Architecture](docs/ARCHITECTURE.md) - System design and patterns
- [Security](docs/SECURITY.md) - Threat model and security measures
- [Crates](docs/CRATES.md) - Detailed crate documentation
- [Interop](docs/INTEROP.md) - Compatibility with official OpenClaw
- [Roadmap](docs/ROADMAP.md) - Implementation progress
- [Contributing](docs/CONTRIBUTING.md) - Development guide

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines.

---

## Acknowledgments

This project is a tribute to the [OpenClaw](https://github.com/openclaw/openclaw) team and community. We're grateful for:

- The excellent design and architecture of the original project
- The vibrant community on Discord
- The open-source spirit that makes projects like this possible

We also build on:
- The Rust ecosystem for its emphasis on safety and performance
- Event sourcing patterns from distributed systems research
- Sandbox techniques from container and browser security

---

## License

MIT License - see [LICENSE](LICENSE) for details.

---

## Legal Notice

- "Claude" and "Anthropic" are trademarks of Anthropic, PBC
- "GPT" and "OpenAI" are trademarks of OpenAI, Inc
- "OpenClaw" is the name of the original open-source project we're inspired by
- All trademarks belong to their respective owners

This is an independent community implementation. Provider integrations use official public APIs.

---

## Links

- [OpenClaw (Official)](https://github.com/openclaw/openclaw) - The original TypeScript project
- [OpenClaw Discord](https://discord.gg/openclaw) - Official community chat
- [OpenClaw Docs](https://docs.openclaw.dev) - Official documentation
