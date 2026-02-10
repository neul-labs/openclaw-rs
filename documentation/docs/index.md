# OpenClaw Rust Core

> **A Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw) by [Neul Labs](https://neullabs.com)**

---

## What is This?

**openclaw-rs** is a Rust implementation of the [OpenClaw](https://github.com/openclaw/openclaw) AI agent framework. This project is developed by [Neul Labs](https://neullabs.com) and aims to provide a high-performance, memory-safe alternative.

This project provides:

- **Multi-Provider Support** - Anthropic Claude, OpenAI GPT, and more
- **Event Sourcing** - Append-only session storage with CRDT projections
- **Secure by Default** - Process sandboxing, encrypted credentials, input validation
- **Node.js Bindings** - Use from JavaScript/TypeScript via napi-rs
- **Web Dashboard** - Vue 3 UI for monitoring and interaction
- **CLI Tool** - Full-featured command-line interface

---

## Why Rust?

| Benefit | Impact |
|---------|--------|
| **Performance** | Sub-millisecond message routing, minimal memory footprint |
| **Safety** | Memory safety without GC, thread safety via ownership |
| **Reliability** | No null pointer exceptions, exhaustive pattern matching |
| **Portability** | Single binary deployment, cross-compilation support |
| **Interop** | napi-rs bindings for Node.js compatibility |

---

## Quick Start

```bash
# Install from crates.io
cargo install openclaw-cli

# Run the setup wizard
openclaw onboard

# Start the gateway
openclaw gateway run
```

[:material-rocket-launch: Get Started](getting-started/index.md){ .md-button .md-button--primary }
[:material-book-open-variant: View Guides](guides/cli.md){ .md-button }

---

## Compatibility

This implementation aims for compatibility with the original [OpenClaw](https://github.com/openclaw/openclaw) project:

| Feature | Status |
|---------|--------|
| Config Format | :white_check_mark: Same `~/.openclaw/openclaw.json` (JSON5) |
| Skills | :white_check_mark: Markdown + YAML frontmatter format |
| Plugins | :white_check_mark: TypeScript plugins via IPC bridge |
| Session Storage | :white_check_mark: Compatible event format |

[:material-swap-horizontal: View Compatibility Guide](compatibility/index.md){ .md-button }

---

## Project Structure

```
openclaw-rs/
├── crates/
│   ├── openclaw-core       # Types, config, events, secrets
│   ├── openclaw-providers  # Anthropic, OpenAI clients
│   ├── openclaw-agents     # Runtime, sandbox, workflow
│   ├── openclaw-gateway    # HTTP/WS server, JSON-RPC
│   ├── openclaw-cli        # Command-line interface
│   └── ...
└── bridge/
    └── openclaw-node       # Node.js bindings (napi-rs)
```

[:material-file-tree: View Architecture](architecture/index.md){ .md-button }

---

## Acknowledgments

This project is inspired by and aims to be compatible with the original [OpenClaw](https://github.com/openclaw/openclaw) project. We're grateful for the excellent design and architecture of the original.

---

## Links

- [:fontawesome-brands-github: Original OpenClaw](https://github.com/openclaw/openclaw) - The original TypeScript project
- [:fontawesome-brands-github: openclaw-rs](https://github.com/neul-labs/openclaw-rs) - This Rust implementation
- [:material-web: Neul Labs](https://neullabs.com) - Organization

---

<small>
**Legal Notice:** "OpenClaw" refers to the original open-source project at [github.com/openclaw/openclaw](https://github.com/openclaw/openclaw). "Claude" and "Anthropic" are trademarks of Anthropic, PBC. "GPT" and "OpenAI" are trademarks of OpenAI, Inc. All trademarks belong to their respective owners. This is an independent implementation by Neul Labs using official public APIs.
</small>
