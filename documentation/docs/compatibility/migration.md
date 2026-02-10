# Migration Guide

!!! info "Community Implementation"
    A Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw) by [Neul Labs](https://neullabs.com).

This guide helps you evaluate and migrate from OpenClaw to openclaw-rs.

---

## Before You Migrate

### Evaluate Your Needs

| If you need... | Recommendation |
|----------------|----------------|
| Maximum compatibility | Stay with OpenClaw |
| Better performance | Consider openclaw-rs |
| Lower resource usage | Consider openclaw-rs |
| All official features | Stay with OpenClaw |
| Rust ecosystem integration | Use openclaw-rs |

### Check Feature Support

Review the [compatibility overview](index.md) to ensure your required features are supported.

---

## Migration Approaches

### 1. Side-by-Side (Recommended)

Run both projects simultaneously during evaluation:

```bash
# OpenClaw on default port
openclaw gateway run  # Port 18789

# openclaw-rs on different port
OPENCLAW_CONFIG=~/.openclaw/openclaw.json \
  openclaw-rs gateway run --port 18790
```

Test with both before fully switching.

### 2. Gradual Migration

Migrate one component at a time:

1. Start with the gateway
2. Then agent configurations
3. Then skills
4. Finally plugins

### 3. Full Switch

For new deployments or when confident:

1. Stop OpenClaw
2. Install openclaw-rs
3. Copy configuration
4. Start openclaw-rs

---

## Step-by-Step Migration

### Step 1: Install openclaw-rs

```bash
cargo install openclaw-cli
```

Verify installation:

```bash
openclaw --version
```

### Step 2: Copy Configuration

```bash
# Create openclaw-rs config directory
mkdir -p ~/.openclaw

# Copy existing config
cp ~/.config/openclaw/openclaw.json ~/.openclaw/openclaw.json

# Or symlink for shared config
ln -s ~/.config/openclaw/openclaw.json ~/.openclaw/openclaw.json
```

### Step 3: Validate Configuration

```bash
openclaw config validate

# Check for compatibility issues
openclaw config validate --compat --verbose
```

Fix any reported issues.

### Step 4: Test Providers

```bash
# Run health checks
openclaw doctor

# Verify API connectivity
openclaw doctor --check providers
```

### Step 5: Migrate Skills

Copy skill definitions:

```bash
cp -r ~/.config/openclaw/skills ~/.openclaw/skills
```

Test each skill:

```bash
openclaw skills list
openclaw skills validate
```

### Step 6: Migrate Plugins (if applicable)

Install the plugin host:

```bash
npm install -g @openclaw-rs/plugin-host
```

Update plugin configuration:

```json5
{
  "plugins": {
    "my_plugin": {
      "path": "/path/to/plugin",
      "enabled": true,
      "bridge": true  // Use IPC bridge
    }
  }
}
```

### Step 7: Start Gateway

```bash
# Start with verbose logging initially
OPENCLAW_LOG=debug openclaw gateway run
```

### Step 8: Verify Functionality

```bash
# Check status
openclaw status

# Run diagnostics
openclaw doctor

# Test via dashboard
open http://localhost:18789
```

---

## Configuration Mapping

### Equivalent Settings

| OpenClaw | openclaw-rs | Notes |
|----------|-------------|-------|
| `server.port` | `gateway.port` | Same |
| `server.host` | `gateway.bind` | Same |
| `llm.providers` | `providers` | Same format |
| `agents` | `agents` | Same format |
| `skills` | `skills` | Same format |

### Changed Settings

| OpenClaw | openclaw-rs | Change |
|----------|-------------|--------|
| `server.workers` | `runtime.worker_threads` | Renamed |
| `sandbox.type` | `sandbox.method` | Different options |

### New Settings

openclaw-rs adds:

```json5
{
  // Rust runtime configuration
  "runtime": {
    "worker_threads": 4,
    "blocking_threads": 512
  },

  // Native sandbox
  "sandbox": {
    "enabled": true,
    "method": "bubblewrap"
  }
}
```

---

## Data Migration

### Sessions

Sessions are stored differently:

```bash
# Export sessions from OpenClaw
openclaw-original sessions export --all > sessions.json

# Import into openclaw-rs (if supported)
openclaw sessions import < sessions.json
```

Or simply start fresh - sessions are typically ephemeral.

### Credentials

Credentials use different encryption:

```bash
# Re-configure authentication
openclaw configure --section auth
```

Enter your API keys again for secure storage.

---

## Troubleshooting Migration

### Config Validation Errors

```bash
# See detailed errors
openclaw config validate --verbose

# Common fix: update deprecated keys
openclaw config migrate  # If available
```

### Provider Connection Issues

```bash
# Test specific provider
openclaw doctor --check providers --verbose

# Check API key
echo $ANTHROPIC_API_KEY | head -c 10
```

### Plugin Loading Failures

```bash
# Enable plugin debug logging
OPENCLAW_LOG=debug,plugin=trace openclaw gateway run --plugins

# Test plugin host directly
npx @openclaw-rs/plugin-host --plugin /path/to/plugin --test
```

### Performance Issues

```bash
# Check resource usage
openclaw status --verbose

# Tune worker threads
openclaw config set runtime.worker_threads 8
```

---

## Rollback Plan

If you need to revert:

1. Stop openclaw-rs:
   ```bash
   openclaw gateway stop
   ```

2. Start OpenClaw:
   ```bash
   openclaw-original gateway run
   ```

Keep your original OpenClaw installation until migration is complete.

---

## Getting Help

### Resources

- [GitHub Issues](https://github.com/neul-labs/openclaw-rs/issues)
- [Compatibility Docs](index.md)

### Official OpenClaw Support

For OpenClaw-specific questions:

- [OpenClaw GitHub](https://github.com/neul-labs/openclaw-rs)
- [OpenClaw Discord](https://neullabs.com)

---

## Post-Migration

### Cleanup

After successful migration:

```bash
# Remove old config (optional)
rm -rf ~/.config/openclaw

# Or keep as backup
mv ~/.config/openclaw ~/.config/openclaw.backup
```

### Optimization

Tune for your workload:

```json5
{
  "runtime": {
    "worker_threads": 8,      // Match CPU cores
    "blocking_threads": 1024  // For I/O heavy workloads
  }
}
```

### Monitoring

Set up monitoring:

```bash
# Enable metrics
openclaw config set gateway.metrics.enabled true

# View metrics
curl http://localhost:18789/metrics
```

---

## Next Steps

[:material-rocket-launch: Quick Start](../getting-started/quick-start.md){ .md-button .md-button--primary }
[:material-code-braces: Contributing](../development/contributing.md){ .md-button }
