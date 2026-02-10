# Development Setup

!!! info "Community Implementation"
    This is a community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw).

Set up your development environment for openclaw-rs.

---

## Prerequisites

### Required

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.85+ | Core development |
| Node.js | 20+ | UI development |
| Git | 2.x | Version control |

### Optional

| Tool | Purpose |
|------|---------|
| Docker | Container testing |
| bubblewrap | Linux sandboxing |
| just | Task runner |

---

## Installation

### Rust Toolchain

```bash
# Install rustup (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install stable toolchain
rustup install stable
rustup default stable

# Add components
rustup component add clippy rustfmt

# Verify
rustc --version
cargo --version
```

### Node.js

```bash
# Using nvm (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 20
nvm use 20

# Verify
node --version
npm --version
```

### Clone Repository

```bash
git clone https://github.com/openclaw/openclaw-rs
cd openclaw-rs
```

---

## Building

### Full Build

```bash
# Build all crates
cargo build

# Release build
cargo build --release
```

### Specific Crates

```bash
# Core only
cargo build -p openclaw-core

# CLI only
cargo build -p openclaw-cli

# Multiple crates
cargo build -p openclaw-core -p openclaw-providers
```

### Web UI

```bash
cd crates/openclaw-ui

# Install dependencies
npm install

# Development server
npm run dev

# Production build
npm run build
```

### Node.js Bindings

```bash
cd bridge/openclaw-node

# Install dependencies
npm install

# Build native module
npm run build

# Debug build
npm run build:debug
```

---

## Running

### Gateway (Development)

```bash
# Run with cargo
cargo run -p openclaw-cli -- gateway run

# With environment
OPENCLAW_LOG=debug cargo run -p openclaw-cli -- gateway run

# With specific config
cargo run -p openclaw-cli -- --config ./test-config.json gateway run
```

### Gateway (Release)

```bash
# Build release
cargo build --release -p openclaw-cli

# Run binary directly
./target/release/openclaw gateway run
```

### UI Development

```bash
# Terminal 1: Start gateway
cargo run -p openclaw-cli -- gateway run

# Terminal 2: Start UI dev server
cd crates/openclaw-ui
npm run dev
```

The dev server proxies API requests to the gateway.

---

## Testing

### Rust Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p openclaw-core

# Specific test
cargo test test_message_creation

# With output
cargo test -- --nocapture

# Integration tests only
cargo test --test '*'
```

### UI Tests

```bash
cd crates/openclaw-ui

# Unit tests
npm run test

# E2E tests
npm run test:e2e
```

### Node.js Bindings Tests

```bash
cd bridge/openclaw-node

# Run tests
npm test
```

---

## Linting

### Rust

```bash
# Clippy (linter)
cargo clippy --all-targets --all-features

# Strict mode
cargo clippy -- -D warnings

# Fix suggestions
cargo clippy --fix
```

### Formatting

```bash
# Check formatting
cargo fmt --check

# Apply formatting
cargo fmt
```

### UI

```bash
cd crates/openclaw-ui

# ESLint
npm run lint

# Fix
npm run lint:fix
```

---

## IDE Setup

### VS Code

Recommended extensions:

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "tamasfe.even-better-toml",
    "serayuzgur.crates",
    "vadimcn.vscode-lldb",
    "Vue.volar"
  ]
}
```

Settings (`.vscode/settings.json`):

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

### IntelliJ/CLion

1. Install Rust plugin
2. Open project as Cargo workspace
3. Enable auto-import

---

## Environment Variables

### Development

```bash
# Logging
export OPENCLAW_LOG=debug

# Test API keys (use test accounts!)
export ANTHROPIC_API_KEY=sk-ant-test-...
export OPENAI_API_KEY=sk-test-...

# Config path
export OPENCLAW_CONFIG=./dev-config.json
```

### .env File

Create `.env` in project root:

```bash
OPENCLAW_LOG=debug
ANTHROPIC_API_KEY=sk-ant-test-...
OPENAI_API_KEY=sk-test-...
```

---

## Project Structure

```
openclaw-rs/
├── Cargo.toml              # Workspace manifest
├── crates/
│   ├── openclaw-core/      # Core types, auth
│   ├── openclaw-providers/ # AI providers
│   ├── openclaw-agents/    # Agent runtime
│   ├── openclaw-channels/  # Pub/sub
│   ├── openclaw-plugins/   # Plugin system
│   ├── openclaw-ipc/       # IPC bridge
│   ├── openclaw-gateway/   # HTTP server
│   ├── openclaw-cli/       # CLI tool
│   └── openclaw-ui/        # Vue 3 UI
├── bridge/
│   └── openclaw-node/      # Node.js bindings
├── documentation/          # MkDocs
└── tests/                  # Integration tests
```

---

## Common Tasks

### Add a Dependency

```bash
# To specific crate
cargo add serde -p openclaw-core

# With features
cargo add tokio -p openclaw-gateway --features full
```

### Create New Crate

```bash
# Create crate
cargo new --lib crates/openclaw-new

# Add to workspace in Cargo.toml
# members = [..., "crates/openclaw-new"]
```

### Generate Docs

```bash
# Rust docs
cargo doc --open

# MkDocs
cd documentation
mkdocs serve
```

---

## Troubleshooting

### Build Errors

```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update
```

### Test Failures

```bash
# Run with backtrace
RUST_BACKTRACE=1 cargo test

# Run specific failing test
cargo test test_name -- --nocapture
```

### Linking Errors (Node.js)

```bash
cd bridge/openclaw-node
rm -rf node_modules
npm install
npm run build
```

---

## Next Steps

[:material-code-braces: Contributing Guide](contributing.md){ .md-button .md-button--primary }
[:material-map: Roadmap](roadmap.md){ .md-button }
