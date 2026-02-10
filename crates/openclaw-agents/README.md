# openclaw-agents

> **Agent runtime for the community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw)**

[![Crates.io](https://img.shields.io/crates/v/openclaw-agents.svg)](https://crates.io/crates/openclaw-agents)
[![Documentation](https://docs.rs/openclaw-agents/badge.svg)](https://docs.rs/openclaw-agents)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

Part of [openclaw-rs](https://github.com/openclaw/openclaw-rs), a community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw). This crate provides agent runtime, sandboxing, and workflow engine.

## Features

- **Agent Runtime**: Execute AI agents with tool access
- **Sandbox**: Platform-specific process isolation
  - Linux: bubblewrap (bwrap)
  - macOS: sandbox-exec
  - Windows: Job Objects
- **Tool Registry**: Register and execute tools
- **Workflows**: Node-based workflow execution (m9m pattern)

## Usage

```rust
use openclaw_agents::{AgentRuntime, ToolRegistry, SandboxConfig, SandboxLevel};
use openclaw_providers::AnthropicProvider;

// Create tool registry
let mut tools = ToolRegistry::new();
tools.register(my_tool);

// Create sandbox config
let sandbox = SandboxConfig {
    level: SandboxLevel::Strict,
    allowed_paths: vec![PathBuf::from("./workspace")],
    network_access: false,
    max_memory_mb: Some(512),
    timeout_seconds: Some(30),
};

// Create agent runtime
let runtime = AgentRuntime::new(provider)
    .with_tools(tools)
    .with_sandbox(sandbox);

// Process message
let response = runtime.process_message(&mut context, "Hello!").await?;
```

## Sandbox Levels

| Level | File Access | Network | Use Case |
|-------|-------------|---------|----------|
| `None` | Full | Allowed | Development/testing |
| `Relaxed` | Workspace only | Allowed | General agents |
| `Strict` | Workspace only | Blocked | Untrusted code execution |

## License

MIT License - see [LICENSE](../../LICENSE) for details.
