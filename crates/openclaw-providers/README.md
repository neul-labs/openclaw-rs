# openclaw-providers

AI provider clients for [OpenClaw](https://github.com/openclaw/openclaw-rs) with full streaming support.

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
