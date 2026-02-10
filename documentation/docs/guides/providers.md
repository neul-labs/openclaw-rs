# AI Provider Configuration

!!! info "Community Implementation"
    This is a community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw).

openclaw-rs supports multiple AI providers through a unified interface.

---

## Supported Providers

| Provider | Status | Models |
|----------|--------|--------|
| **Anthropic** | Full support | Claude 3.5, Claude 3 |
| **OpenAI** | Full support | GPT-4o, GPT-4, GPT-3.5 |

---

## Anthropic

### Configuration

```json5
{
  "providers": {
    "anthropic": {
      "api_key_env": "ANTHROPIC_API_KEY",
      "default_model": "claude-3-5-sonnet-20241022",
      "base_url": "https://api.anthropic.com"  // optional
    }
  }
}
```

### API Key Setup

1. Get your API key from [Anthropic Console](https://console.anthropic.com/)
2. Set the environment variable:

```bash
export ANTHROPIC_API_KEY="sk-ant-api03-..."
```

### Available Models

| Model ID | Description |
|----------|-------------|
| `claude-3-5-sonnet-20241022` | Latest Sonnet - fast, capable |
| `claude-3-opus-20240229` | Most capable |
| `claude-3-sonnet-20240229` | Balanced |
| `claude-3-haiku-20240307` | Fastest |

### Streaming

Anthropic supports streaming responses via Server-Sent Events (SSE):

```rust
// Rust example
let stream = provider.stream_message(&request).await?;
while let Some(chunk) = stream.next().await {
    match chunk? {
        StreamChunk::ContentDelta(text) => print!("{}", text),
        StreamChunk::Done => break,
    }
}
```

---

## OpenAI

### Configuration

```json5
{
  "providers": {
    "openai": {
      "api_key_env": "OPENAI_API_KEY",
      "default_model": "gpt-4o",
      "org_id": "org-...",  // optional
      "base_url": "https://api.openai.com/v1"  // optional
    }
  }
}
```

### API Key Setup

1. Get your API key from [OpenAI Platform](https://platform.openai.com/)
2. Set the environment variable:

```bash
export OPENAI_API_KEY="sk-..."
```

### Available Models

| Model ID | Description |
|----------|-------------|
| `gpt-4o` | Latest GPT-4 Omni |
| `gpt-4o-mini` | Smaller, faster GPT-4o |
| `gpt-4-turbo` | GPT-4 Turbo |
| `gpt-4` | Original GPT-4 |
| `gpt-3.5-turbo` | Fast, economical |

### Organization ID

For organization accounts, set the org ID:

```json5
{
  "providers": {
    "openai": {
      "api_key_env": "OPENAI_API_KEY",
      "org_id": "org-abc123..."
    }
  }
}
```

---

## Custom Base URLs

Both providers support custom base URLs for:

- Proxy servers
- Self-hosted models
- API-compatible alternatives

```json5
{
  "providers": {
    "openai": {
      "api_key_env": "OPENAI_API_KEY",
      "base_url": "https://my-proxy.example.com/v1"
    }
  }
}
```

---

## Provider Selection

### Default Provider

Set the default provider for agents:

```json5
{
  "agents": {
    "default": {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022"
    }
  }
}
```

### Per-Agent Override

Each agent can use a different provider:

```json5
{
  "agents": {
    "default": {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022"
    },
    "fast-responder": {
      "provider": "openai",
      "model": "gpt-4o-mini"
    },
    "deep-thinker": {
      "provider": "anthropic",
      "model": "claude-3-opus-20240229"
    }
  }
}
```

---

## Secure Credential Storage

For production deployments, use the encrypted credential store:

```bash
# Store credentials securely
openclaw configure --section auth
```

This encrypts API keys using AES-256-GCM with a master key.

### How It Works

1. Master key derived from system-specific data
2. API keys encrypted at rest in `~/.openclaw/credentials/`
3. Keys decrypted only when needed
4. Memory cleared after use

---

## Error Handling

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `401 Unauthorized` | Invalid API key | Check your API key |
| `429 Rate Limited` | Too many requests | Implement backoff |
| `500 Server Error` | Provider issue | Retry with backoff |

### Rate Limiting

openclaw-rs includes automatic retry with exponential backoff:

```json5
{
  "providers": {
    "anthropic": {
      "api_key_env": "ANTHROPIC_API_KEY",
      "retry": {
        "max_attempts": 3,
        "initial_delay_ms": 1000,
        "max_delay_ms": 30000
      }
    }
  }
}
```

---

## Testing Providers

Verify provider connectivity:

```bash
# Check all providers
openclaw doctor

# Test specific provider
openclaw status --verbose
```

---

## Next Steps

[:material-robot: Agent Configuration](agents.md){ .md-button .md-button--primary }
[:material-cog: Full Configuration Reference](../getting-started/configuration.md){ .md-button }
