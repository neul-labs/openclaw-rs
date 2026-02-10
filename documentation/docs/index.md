# OpenClaw Rust Core

> **A community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw)**

!!! info "Community Implementation"
    This is an **unofficial, community-driven** Rust implementation of OpenClaw.

    **For the official project, please visit:**

    - [:fontawesome-brands-github: OpenClaw on GitHub](https://github.com/openclaw/openclaw) - The official TypeScript implementation
    - [:fontawesome-brands-discord: OpenClaw Discord](https://discord.gg/openclaw) - Official community chat
    - [:material-book: OpenClaw Docs](https://docs.openclaw.dev) - Official documentation

---

## What is This?

**openclaw-rs** is a tribute to the excellent [OpenClaw](https://github.com/openclaw/openclaw) project, reimplemented in Rust. We love OpenClaw and wanted to explore what a high-performance Rust implementation might look like.

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

This implementation aims for compatibility with the official OpenClaw project:

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

This project is a tribute to the [OpenClaw](https://github.com/openclaw/openclaw) team and community. We're grateful for the excellent design and architecture of the original project.

---

<small>
**Legal Notice:** "Claude" and "Anthropic" are trademarks of Anthropic, PBC. "GPT" and "OpenAI" are trademarks of OpenAI, Inc. "OpenClaw" refers to the original open-source project we're inspired by. All trademarks belong to their respective owners. This is an independent community implementation using official public APIs.
</small>
