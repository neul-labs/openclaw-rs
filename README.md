# OpenClaw Rust Core

High-performance Rust core for [OpenClaw](https://github.com/openclaw/openclaw).

![Rust](https://img.shields.io/badge/rust-1.85%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## Why Rust?

OpenClaw's TypeScript implementation serves well for rapid iteration, but as the project matures, certain components benefit from Rust's guarantees:

- **Performance**: Sub-millisecond message routing, minimal memory footprint
- **Safety**: Memory safety without GC, thread safety via ownership
- **Reliability**: No null pointer exceptions, exhaustive pattern matching
- **Portability**: Single binary deployment, cross-compilation to mobile/embedded
- **Interop**: napi-rs bindings preserve the existing TypeScript ecosystem

## Architecture

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
| `openclaw-core` | ✅ Complete | Types, config, events, secrets, auth, validation |
| `openclaw-ipc` | ✅ Complete | IPC message types, nng transport |
| `openclaw-providers` | ⚠️ Partial | Provider traits, API client stubs |
| `openclaw-agents` | ✅ Complete | Runtime, sandbox, workflow, tools |
| `openclaw-channels` | ⚠️ Partial | Channel traits, routing, allowlist |
| `openclaw-gateway` | ⚠️ Partial | HTTP/WS server, JSON-RPC |
| `openclaw-plugins` | ⚠️ Partial | Plugin API, FFI bridge |
| `openclaw-cli` | ⚠️ Partial | CLI commands |
| `openclaw-node` | 🔜 Planned | napi-rs bindings for Node.js |

## Installation

### From Source

```bash
# Clone and build
git clone https://github.com/openclaw/openclaw-rs
cd openclaw-rs
cargo build --release

# Install the CLI
cargo install --path crates/openclaw-cli
```

### Requirements

- Rust 1.85+ (2024 edition)
- System dependencies for sandboxing:
  - Linux: `bubblewrap` (`bwrap`)
  - macOS: Built-in `sandbox-exec`
  - Windows: No additional deps (uses Job Objects)

## Getting Started

After installation, run the onboarding wizard:

```bash
# Interactive setup wizard
openclaw onboard

# Or for automation:
openclaw onboard --non-interactive --accept-risk --flow quickstart --auth-choice anthropic --api-key "sk-..."
```

The wizard will:
1. Accept security acknowledgement
2. Configure gateway (port, bind address)
3. Set up AI provider authentication
4. Create workspace directory
5. Optionally install as system service

### Post-Setup Commands

```bash
# Check status
openclaw status

# Start gateway
openclaw gateway run

# Run health checks
openclaw doctor

# Install shell completion
openclaw completion --install
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `openclaw onboard` | Interactive setup wizard |
| `openclaw configure` | Update configuration interactively |
| `openclaw doctor` | Health checks with auto-repair |
| `openclaw status` | Show gateway/channels status |
| `openclaw gateway run` | Start the gateway server |
| `openclaw config get/set` | Configuration management |
| `openclaw completion` | Shell completion setup |
| `openclaw daemon` | System service management |
| `openclaw reset` | Reset configuration/state |

## Building

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run with release optimizations
cargo build --workspace --release

# Generate documentation
cargo doc --workspace --open

# Run the CLI (dev)
cargo run -p openclaw-cli -- --help
```

## Design Principles

| Principle | Implementation |
|-----------|----------------|
| **Event Sourcing** | Sessions stored as append-only event logs (sled) |
| **CRDT Projections** | Conflict-free state via last-write-wins merge |
| **Defense in Depth** | Input validation, sandboxing, secret redaction |
| **Zero Trust** | All external input validated at boundaries |
| **Fail Secure** | Errors default to denial, not exposure |

## Relationship to OpenClaw (TypeScript)

This Rust implementation is designed to **complement**, not replace, the TypeScript ecosystem:

- **Skills**: Continue using Markdown + YAML frontmatter format
- **Plugins**: TypeScript plugins communicate via IPC bridge
- **Config**: Same `~/.openclaw/openclaw.json` format (JSON5)
- **Migration**: Gradual, component-by-component replacement

See [docs/INTEROP.md](docs/INTEROP.md) for the full interoperability strategy.

## Documentation

- [Architecture](docs/ARCHITECTURE.md) - System design and patterns
- [Security](docs/SECURITY.md) - Threat model and security measures
- [Roadmap](docs/ROADMAP.md) - Implementation progress and plans
- [Interop](docs/INTEROP.md) - TypeScript compatibility guide
- [Crates](docs/CRATES.md) - Detailed crate documentation
- [Contributing](docs/CONTRIBUTING.md) - Development guide

## License

MIT License - see [LICENSE](LICENSE) for details.

## See Also

- [OpenClaw (TypeScript)](https://github.com/openclaw/openclaw) - Main project
- [grite](https://github.com/anthropics/grite) - Event sourcing patterns
- [m9m](https://github.com/anthropics/m9m) - Workflow node patterns
