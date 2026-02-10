# Configuration

!!! info "Community Implementation"
    This is a community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw).

openclaw-rs uses the same configuration format as the official OpenClaw project, ensuring compatibility.

## Configuration File

The main configuration file is located at:

```
~/.openclaw/openclaw.json
```

This is a JSON5 file (JSON with comments and trailing commas allowed).

---

## Basic Structure

```json5
{
  // Gateway settings
  "gateway": {
    "port": 18789,
    "bind": "127.0.0.1",
    "cors_origins": ["http://localhost:3000"]
  },

  // AI provider configuration
  "providers": {
    "anthropic": {
      "api_key_env": "ANTHROPIC_API_KEY",
      "default_model": "claude-3-5-sonnet-20241022"
    },
    "openai": {
      "api_key_env": "OPENAI_API_KEY",
      "default_model": "gpt-4o"
    }
  },

  // Agent configuration
  "agents": {
    "default": {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022",
      "system_prompt": "You are a helpful assistant.",
      "max_tokens": 4096
    }
  },

  // Workspace settings
  "workspace": {
    "path": "~/.openclaw/workspace",
    "allowed_paths": ["~/projects"]
  }
}
```

---

## Configuration Sections

### Gateway

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `port` | number | `18789` | HTTP server port |
| `bind` | string | `"127.0.0.1"` | Bind address |
| `cors_origins` | string[] | `[]` | Allowed CORS origins |
| `tls.cert` | string | - | TLS certificate path |
| `tls.key` | string | - | TLS key path |

### Providers

Each provider can have:

| Field | Type | Description |
|-------|------|-------------|
| `api_key_env` | string | Environment variable containing API key |
| `api_key` | string | API key (prefer env var for security) |
| `base_url` | string | Custom API endpoint |
| `default_model` | string | Default model to use |
| `org_id` | string | Organization ID (OpenAI) |

### Agents

| Field | Type | Description |
|-------|------|-------------|
| `provider` | string | Provider name (`anthropic`, `openai`) |
| `model` | string | Model identifier |
| `system_prompt` | string | System prompt for the agent |
| `max_tokens` | number | Maximum tokens per response |
| `temperature` | number | Sampling temperature (0-1) |
| `tools` | string[] | Enabled tools |

---

## Environment Variables

API keys should be stored in environment variables:

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
export OPENAI_API_KEY="sk-..."
```

Add to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.) for persistence.

---

## Managing Configuration

### View Current Config

```bash
openclaw config show
```

### Get a Value

```bash
openclaw config get gateway.port
```

### Set a Value

```bash
openclaw config set gateway.port 8080
```

### Validate Config

```bash
openclaw config validate
```

### Interactive Configuration

```bash
openclaw configure
```

---

## Secure Credential Storage

For enhanced security, use the encrypted credential store:

```bash
# Store credentials securely
openclaw configure --section auth
```

This encrypts API keys using AES-256-GCM and stores them in `~/.openclaw/credentials/`.

---

## Multiple Environments

You can use different config files:

```bash
# Use a specific config
openclaw --config ~/configs/production.json gateway run

# Or set via environment
export OPENCLAW_CONFIG=~/configs/production.json
openclaw gateway run
```

---

## Compatibility Note

!!! note "OpenClaw Compatibility"
    This configuration format is compatible with the official [OpenClaw](https://github.com/openclaw/openclaw) project. You can share configuration files between implementations.

---

## Next Steps

[:material-console: CLI Usage Guide](../guides/cli.md){ .md-button .md-button--primary }
[:material-robot: Provider Configuration](../guides/providers.md){ .md-button }
