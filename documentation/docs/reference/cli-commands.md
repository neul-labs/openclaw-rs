# CLI Command Reference

!!! info "Community Implementation"
    This is a community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw).

Complete reference for all `openclaw` CLI commands.

---

## Synopsis

```
openclaw [OPTIONS] <COMMAND>
```

---

## Global Options

| Option | Short | Description |
|--------|-------|-------------|
| `--config <PATH>` | `-c` | Use specific config file |
| `--verbose` | `-v` | Enable verbose output |
| `--quiet` | `-q` | Suppress non-error output |
| `--help` | `-h` | Show help |
| `--version` | `-V` | Show version |

---

## onboard

Interactive setup wizard.

### Synopsis

```
openclaw onboard [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--non-interactive` | Skip interactive prompts |
| `--accept-risk` | Accept security acknowledgment |
| `--flow <FLOW>` | Setup flow: `quickstart`, `full` |
| `--auth-choice <PROVIDER>` | Provider: `anthropic`, `openai`, `both` |
| `--api-key <KEY>` | API key for provider |
| `--skip-service` | Skip service installation |

### Examples

```bash
# Interactive setup
openclaw onboard

# Automated setup
openclaw onboard \
  --non-interactive \
  --accept-risk \
  --flow quickstart \
  --auth-choice anthropic \
  --api-key "sk-ant-..."

# Full setup with both providers
openclaw onboard --flow full --auth-choice both
```

---

## gateway

Gateway server management.

### gateway run

Start the gateway server.

```
openclaw gateway run [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--port <PORT>` | Override configured port |
| `--bind <ADDRESS>` | Override bind address |
| `--foreground` | Run in foreground (default) |

```bash
# Start with defaults
openclaw gateway run

# Custom port
openclaw gateway run --port 8080

# Bind to all interfaces
openclaw gateway run --bind 0.0.0.0
```

### gateway stop

Stop the running gateway.

```
openclaw gateway stop
```

### gateway restart

Restart the gateway.

```
openclaw gateway restart
```

### gateway status

Show gateway status.

```
openclaw gateway status [--json]
```

---

## config

Configuration management.

### config show

Display current configuration.

```
openclaw config show [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--format <FORMAT>` | Output format: `json`, `yaml`, `table` |
| `--section <SECTION>` | Show specific section |

```bash
# Show all
openclaw config show

# JSON format
openclaw config show --format json

# Specific section
openclaw config show --section gateway
```

### config get

Get a configuration value.

```
openclaw config get <KEY>
```

```bash
openclaw config get gateway.port
# Output: 18789

openclaw config get providers.anthropic.default_model
# Output: claude-3-5-sonnet-20241022
```

### config set

Set a configuration value.

```
openclaw config set <KEY> <VALUE>
```

```bash
openclaw config set gateway.port 8080
openclaw config set providers.openai.default_model gpt-4o
```

### config validate

Validate configuration file.

```
openclaw config validate [--strict]
```

```bash
openclaw config validate
# Output: Configuration is valid

openclaw config validate --strict
# Checks for warnings too
```

### config path

Show configuration file path.

```
openclaw config path
```

---

## configure

Interactive configuration editor.

```
openclaw configure [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--section <SECTION>` | Configure section: `gateway`, `auth`, `agents`, `all` |

```bash
# Configure authentication
openclaw configure --section auth

# Full configuration
openclaw configure --section all
```

---

## status

Show system status.

```
openclaw status [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON |
| `--watch` | Continuously update |
| `--interval <SECS>` | Watch interval (default: 2) |

```bash
openclaw status

# Example output:
# OpenClaw Status
# ===============
# Gateway:     Running (http://127.0.0.1:18789)
# Uptime:      2h 34m
# Sessions:    3 active
# Providers:   anthropic (ok), openai (ok)
# Memory:      45.2 MB
# CPU:         0.1%

openclaw status --json
openclaw status --watch
```

---

## doctor

Run health checks.

```
openclaw doctor [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--fix` | Attempt to fix issues |
| `--verbose` | Show detailed output |
| `--check <CHECK>` | Run specific check |

```bash
openclaw doctor

# Example output:
# Health Check Results
# ====================
# ✓ Configuration file exists
# ✓ Configuration is valid
# ✓ Anthropic API accessible
# ✓ OpenAI API accessible
# ✓ Gateway is running
# ✓ Workspace exists and writable
# ✓ Required dependencies installed
#
# All checks passed!

openclaw doctor --fix
openclaw doctor --verbose
```

### Available Checks

| Check | Description |
|-------|-------------|
| `config` | Configuration validity |
| `providers` | Provider API connectivity |
| `gateway` | Gateway availability |
| `workspace` | Workspace permissions |
| `deps` | System dependencies |

---

## sessions

Session management.

### sessions list

List all sessions.

```
openclaw sessions list [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--limit <N>` | Limit results |
| `--offset <N>` | Skip first N results |
| `--active` | Show only active |
| `--json` | Output as JSON |
| `--sort <FIELD>` | Sort by: `created`, `updated`, `messages` |

```bash
openclaw sessions list
openclaw sessions list --limit 10 --active
openclaw sessions list --json
```

### sessions show

Show session details.

```
openclaw sessions show <SESSION_ID> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON |
| `--messages` | Include all messages |

```bash
openclaw sessions show abc123
openclaw sessions show abc123 --messages
```

### sessions delete

Delete a session.

```
openclaw sessions delete <SESSION_ID> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--force` | Skip confirmation |

```bash
openclaw sessions delete abc123
openclaw sessions delete abc123 --force
```

### sessions export

Export a session.

```
openclaw sessions export <SESSION_ID> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--format <FORMAT>` | Format: `json`, `markdown`, `text` |
| `--output <PATH>` | Output file path |

```bash
openclaw sessions export abc123 --format markdown
openclaw sessions export abc123 --format json --output session.json
```

### sessions clear

Clear all sessions.

```
openclaw sessions clear [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--force` | Skip confirmation |
| `--older-than <DAYS>` | Only clear older sessions |

```bash
openclaw sessions clear --older-than 30
```

---

## agents

Agent management.

### agents list

List configured agents.

```
openclaw agents list [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON |

### agents show

Show agent configuration.

```
openclaw agents show <AGENT_NAME>
```

### agents create

Create a new agent.

```
openclaw agents create <AGENT_NAME> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--provider <PROVIDER>` | AI provider |
| `--model <MODEL>` | Model to use |
| `--system-prompt <PROMPT>` | System prompt |
| `--interactive` | Interactive creation |

---

## tools

Tool management.

### tools list

List available tools.

```
openclaw tools list [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON |
| `--enabled` | Show only enabled |

### tools show

Show tool details.

```
openclaw tools show <TOOL_NAME>
```

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `OPENCLAW_CONFIG` | Config file path |
| `OPENCLAW_LOG` | Log level: `debug`, `info`, `warn`, `error` |
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `OPENAI_API_KEY` | OpenAI API key |
| `NO_COLOR` | Disable colored output |

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 3 | Connection error |
| 4 | Authentication error |
| 5 | Not found |
| 126 | Permission denied |
| 130 | Interrupted (Ctrl+C) |

---

## Next Steps

[:material-swap-horizontal: OpenClaw Compatibility](../compatibility/index.md){ .md-button .md-button--primary }
[:material-rocket-launch: Quick Start](../getting-started/quick-start.md){ .md-button }
