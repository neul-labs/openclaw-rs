# Configuration Compatibility

!!! info "Community Implementation"
    This is a community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw).

openclaw-rs uses the same configuration format as the official OpenClaw project.

---

## File Format

Both projects use JSON5 (JSON with comments and trailing commas):

```json5
{
  // This is a comment
  "gateway": {
    "port": 18789,
    "bind": "127.0.0.1",  // Trailing comma allowed
  }
}
```

---

## Configuration Location

| Project | Default Path |
|---------|--------------|
| OpenClaw | `~/.config/openclaw/openclaw.json` |
| openclaw-rs | `~/.openclaw/openclaw.json` |

You can use either location:

```bash
# Use OpenClaw's config
openclaw --config ~/.config/openclaw/openclaw.json gateway run

# Or symlink
ln -s ~/.config/openclaw/openclaw.json ~/.openclaw/openclaw.json
```

---

## Shared Configuration

### Fully Compatible

These sections are fully compatible between projects:

#### gateway

```json5
{
  "gateway": {
    "port": 18789,
    "bind": "127.0.0.1",
    "cors_origins": ["http://localhost:3000"],
    "tls": {
      "cert": "/path/to/cert.pem",
      "key": "/path/to/key.pem"
    }
  }
}
```

#### providers

```json5
{
  "providers": {
    "anthropic": {
      "api_key_env": "ANTHROPIC_API_KEY",
      "default_model": "claude-3-5-sonnet-20241022",
      "base_url": "https://api.anthropic.com"
    },
    "openai": {
      "api_key_env": "OPENAI_API_KEY",
      "default_model": "gpt-4o",
      "org_id": "org-..."
    }
  }
}
```

#### agents

```json5
{
  "agents": {
    "default": {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022",
      "system_prompt": "You are helpful.",
      "max_tokens": 4096,
      "temperature": 0.7,
      "tools": ["file_read", "bash"]
    }
  }
}
```

#### workspace

```json5
{
  "workspace": {
    "path": "~/.openclaw/workspace",
    "allowed_paths": ["~/projects", "~/code"]
  }
}
```

---

### Partially Compatible

These sections may have differences:

#### skills

```json5
{
  "skills": {
    // Basic skill definitions work
    "custom_skill": {
      "description": "A custom skill",
      "tool": "bash",
      "command": "echo 'hello'"
    }
  }
}
```

⚠️ Advanced skill features may not be fully supported yet.

#### plugins

```json5
{
  "plugins": {
    "my_plugin": {
      "path": "/path/to/plugin",
      "enabled": true
    }
  }
}
```

⚠️ Plugin loading works, but some plugin APIs may differ.

---

### openclaw-rs Specific

These options are specific to openclaw-rs:

```json5
{
  // Rust-specific performance tuning
  "runtime": {
    "worker_threads": 4,
    "blocking_threads": 512
  },

  // Native sandbox settings
  "sandbox": {
    "enabled": true,
    "method": "bubblewrap"  // Linux only
  }
}
```

---

## Validation

Validate your configuration:

```bash
# openclaw-rs validation
openclaw config validate

# Check for compatibility issues
openclaw config validate --compat
```

---

## Environment Variables

Both projects support the same environment variables:

| Variable | Description |
|----------|-------------|
| `OPENCLAW_CONFIG` | Config file path |
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `OPENAI_API_KEY` | OpenAI API key |

---

## Migrating Configuration

### From OpenClaw to openclaw-rs

1. Copy your config:
   ```bash
   cp ~/.config/openclaw/openclaw.json ~/.openclaw/openclaw.json
   ```

2. Validate:
   ```bash
   openclaw config validate
   ```

3. Fix any warnings:
   ```bash
   openclaw config validate --verbose
   ```

### Sharing Between Both

Use a symlink to share configuration:

```bash
# Create shared location
mkdir -p ~/.config/openclaw

# Link openclaw-rs to use shared config
ln -s ~/.config/openclaw/openclaw.json ~/.openclaw/openclaw.json
```

---

## Known Differences

| Feature | OpenClaw | openclaw-rs |
|---------|----------|-------------|
| Config path default | `~/.config/openclaw/` | `~/.openclaw/` |
| Sandbox method | Node.js based | Native (bwrap, sandbox-exec) |
| Plugin runtime | Node.js | IPC bridge to Node.js |

---

## Next Steps

[:material-puzzle: Skills Compatibility](skills.md){ .md-button .md-button--primary }
[:material-power-plug: Plugin Compatibility](plugins.md){ .md-button }
