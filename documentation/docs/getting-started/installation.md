# Installation

!!! info "Community Implementation"
    This is a community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw).

## Requirements

### System Requirements

- **Rust 1.85+** (2024 edition)
- **Node.js 20+** (for building the embedded UI)

### Platform-Specific Dependencies

=== "Linux"

    ```bash
    # For sandboxing support
    sudo apt install bubblewrap  # Debian/Ubuntu
    sudo dnf install bubblewrap  # Fedora
    sudo pacman -S bubblewrap    # Arch
    ```

=== "macOS"

    No additional dependencies required. Uses built-in `sandbox-exec`.

=== "Windows"

    No additional dependencies required. Uses Windows Job Objects.

---

## Installation Methods

### From crates.io (Recommended)

```bash
cargo install openclaw-cli
```

This installs the `openclaw` binary to `~/.cargo/bin/`.

### From Source

```bash
# Clone the repository
git clone https://github.com/openclaw/openclaw-rs
cd openclaw-rs

# Build in release mode
cargo build --release

# Install the CLI
cargo install --path crates/openclaw-cli
```

### Using the Install Script

```bash
# One-liner install
curl -fsSL https://raw.githubusercontent.com/openclaw/openclaw-rs/main/install.sh | bash

# Or with options
./install.sh --method source    # Force source build
./install.sh --prefix ~/.local  # Custom install prefix
```

The install script tries these methods in order:

1. Pre-built binaries from GitHub Releases
2. `cargo install` from crates.io
3. Build from source (if in repo directory)
4. Clone and build from source

---

## Verify Installation

```bash
# Check version
openclaw --version

# Show help
openclaw --help
```

Expected output:

```
openclaw 0.1.0
Command-line interface for OpenClaw

USAGE:
    openclaw <COMMAND>

COMMANDS:
    onboard     Interactive setup wizard
    gateway     Gateway server management
    status      Show system status
    ...
```

---

## Building the Web UI

The web dashboard is built with Vue 3 and embedded in the gateway:

```bash
cd crates/openclaw-ui
npm install
npm run build
```

The built assets are automatically embedded when you build the gateway.

---

## Next Steps

[:material-rocket-launch: Quick Start Guide](quick-start.md){ .md-button .md-button--primary }
