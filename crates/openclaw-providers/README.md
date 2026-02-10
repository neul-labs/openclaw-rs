# openclaw-providers

> **AI provider clients for the community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw)**

[![Crates.io](https://img.shields.io/crates/v/openclaw-providers.svg)](https://crates.io/crates/openclaw-providers)
[![Documentation](https://docs.rs/openclaw-providers/badge.svg)](https://docs.rs/openclaw-providers)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

Part of [openclaw-rs](https://github.com/openclaw/openclaw-rs), a community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw). This crate provides AI provider clients with full streaming support.

Provider integrations use official public APIs. "Claude" is a trademark of Anthropic, "GPT" is a trademark of OpenAI.

## Supported Providers

- **Anthropic Claude**: Claude 3.5 Sonnet, Claude 3.5 Haiku, and other Claude models
- **OpenAI GPT**: GPT-4o, GPT-4, GPT-3.5, and compatible APIs (Azure, LocalAI)

## Features

- Async/await API with tokio
- Server-Sent Events (SSE) streaming
- Tool/function calling support
- Token usage tracking
- Configurable base URLs for proxies

## Usage

```rust
use openclaw_providers::{AnthropicProvider, Provider, CompletionRequest, Message, Role};
use openclaw_core::secrets::ApiKey;

// Create provider
let provider = AnthropicProvider::new(ApiKey::new("sk-ant-...".to_string()));

// Create completion request
let request = CompletionRequest {
    model: "claude-3-5-sonnet-20241022".to_string(),
    messages: vec![
        Message {
            role: Role::User,
            content: MessageContent::Text("Hello!".to_string()),
        }
    ],
    max_tokens: 1024,
    ..Default::default()
};

// Get completion
let response = provider.complete(request).await?;
println!("{}", response.content);

// Streaming
let mut stream = provider.complete_stream(request).await?;
while let Some(chunk) = stream.next().await {
    if let Some(delta) = chunk?.delta {
        print!("{}", delta);
    }
}
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.
