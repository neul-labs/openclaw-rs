# Getting Started

!!! info "Community Implementation"
    This is a community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw).

Welcome to openclaw-rs! This guide will help you get up and running quickly.

## Overview

openclaw-rs provides a high-performance Rust implementation of the OpenClaw AI agent framework. It includes:

- **CLI** - Command-line interface for managing agents and the gateway
- **Gateway** - HTTP/WebSocket server with JSON-RPC API
- **Web Dashboard** - Vue 3 UI for monitoring and interaction
- **Node.js Bindings** - Use from JavaScript/TypeScript applications

## What You'll Need

Before starting, ensure you have:

- **Rust 1.85+** (2024 edition)
- **Node.js 20+** (for building the web UI)
- An API key from [Anthropic](https://console.anthropic.com/) or [OpenAI](https://platform.openai.com/)

## Quick Path

```bash
# 1. Install
cargo install openclaw-cli

# 2. Setup
openclaw onboard

# 3. Run
openclaw gateway run
```

## Next Steps

<div class="grid cards" markdown>

-   :material-download:{ .lg .middle } **Installation**

    ---

    Install from crates.io or build from source

    [:octicons-arrow-right-24: Install Guide](installation.md)

-   :material-rocket-launch:{ .lg .middle } **Quick Start**

    ---

    Run the setup wizard and start your first agent

    [:octicons-arrow-right-24: Quick Start](quick-start.md)

-   :material-cog:{ .lg .middle } **Configuration**

    ---

    Configure providers, agents, and channels

    [:octicons-arrow-right-24: Configuration](configuration.md)

</div>
