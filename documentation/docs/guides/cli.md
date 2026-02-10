# CLI Usage Guide

!!! info "Community Implementation"
    This is a community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw).

The `openclaw` CLI provides commands for managing the gateway, configuration, and agents.

---

## Global Options

```bash
openclaw [OPTIONS] <COMMAND>
```

| Option | Description |
|--------|-------------|
| `--config <PATH>` | Use a specific config file |
| `--verbose`, `-v` | Enable verbose output |
| `--quiet`, `-q` | Suppress non-error output |
| `--help`, `-h` | Show help |
| `--version`, `-V` | Show version |

---

## Commands Overview

| Command | Description |
|---------|-------------|
| `onboard` | Interactive setup wizard |
| `gateway` | Gateway server management |
| `config` | Configuration management |
| `status` | Show system status |
| `doctor` | Run health checks |
| `sessions` | Session management |

---

## onboard

Interactive setup wizard for first-time configuration.

```bash
openclaw onboard [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--non-interactive` | Skip interactive prompts |
| `--accept-risk` | Accept security notice |
| `--flow <FLOW>` | Setup flow (`quickstart`, `full`) |
| `--auth-choice <PROVIDER>` | Provider choice (`anthropic`, `openai`) |
| `--api-key <KEY>` | API key for the provider |

### Examples

```bash
# Interactive setup
openclaw onboard

# Non-interactive for CI/CD
openclaw onboard \
  --non-interactive \
  --accept-risk \
  --flow quickstart \
  --auth-choice anthropic \
  --api-key "sk-ant-..."
```

---

## gateway

Manage the gateway server.

### gateway run

Start the gateway server.

```bash
openclaw gateway run [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--port <PORT>` | Override port (default: 18789) |
| `--bind <ADDRESS>` | Override bind address |
| `--foreground` | Run in foreground (default) |

### gateway stop

Stop a running gateway.

```bash
openclaw gateway stop
```

### gateway restart

Restart the gateway.

```bash
openclaw gateway restart
```

### Examples

```bash
# Start on custom port
openclaw gateway run --port 8080

# Start in background (via systemd)
sudo systemctl start openclaw
```

---

## config

Manage configuration.

### config show

Display current configuration.

```bash
openclaw config show [--format <FORMAT>]
```

| Format | Description |
|--------|-------------|
| `json` | JSON output (default) |
| `yaml` | YAML output |
| `table` | Table format |

### config get

Get a specific configuration value.

```bash
openclaw config get <KEY>
```

```bash
# Examples
openclaw config get gateway.port
openclaw config get providers.anthropic.default_model
```

### config set

Set a configuration value.

```bash
openclaw config set <KEY> <VALUE>
```

```bash
# Examples
openclaw config set gateway.port 8080
openclaw config set providers.openai.default_model gpt-4o
```

### config validate

Validate the configuration file.

```bash
openclaw config validate
```

---

## status

Show system status.

```bash
openclaw status [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON |
| `--watch` | Continuously update |

Example output:

```
OpenClaw Status
===============
Gateway:     Running (http://127.0.0.1:18789)
Uptime:      2h 34m
Sessions:    3 active
Providers:   anthropic (ok), openai (ok)
```

---

## doctor

Run health checks and diagnostics.

```bash
openclaw doctor [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--fix` | Attempt to fix issues |
| `--verbose` | Show detailed output |

Checks include:

- Configuration file validity
- Provider API connectivity
- Gateway availability
- Workspace permissions
- System dependencies

---

## sessions

Manage conversation sessions.

### sessions list

List all sessions.

```bash
openclaw sessions list [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--limit <N>` | Limit results |
| `--active` | Show only active sessions |
| `--json` | Output as JSON |

### sessions show

Show session details.

```bash
openclaw sessions show <SESSION_ID>
```

### sessions delete

Delete a session.

```bash
openclaw sessions delete <SESSION_ID>
```

---

## configure

Interactive configuration editor.

```bash
openclaw configure [--section <SECTION>]
```

| Section | Description |
|---------|-------------|
| `gateway` | Gateway settings |
| `auth` | Provider credentials |
| `agents` | Agent configuration |
| `all` | Full configuration |

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `OPENCLAW_CONFIG` | Path to config file |
| `OPENCLAW_LOG` | Log level (`debug`, `info`, `warn`, `error`) |
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `OPENAI_API_KEY` | OpenAI API key |

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 3 | Connection error |
| 4 | Authentication error |

---

## Next Steps

[:material-robot: Provider Configuration](providers.md){ .md-button .md-button--primary }
[:material-cog: Configuration Reference](../getting-started/configuration.md){ .md-button }
