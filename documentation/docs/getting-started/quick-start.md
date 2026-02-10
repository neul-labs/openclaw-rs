# Quick Start

!!! info "Community Implementation"
    A Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw) by [Neul Labs](https://neullabs.com).

This guide will get you from zero to a running OpenClaw gateway in about 5 minutes.

## Prerequisites

- [openclaw-cli installed](installation.md)
- An API key from [Anthropic](https://console.anthropic.com/) or [OpenAI](https://platform.openai.com/)

---

## Step 1: Run the Setup Wizard

The onboarding wizard guides you through initial configuration:

```bash
openclaw onboard
```

The wizard will:

1. **Security Acknowledgement** - Accept the security notice
2. **Gateway Configuration** - Set port and bind address
3. **Provider Setup** - Configure Anthropic or OpenAI
4. **Workspace Creation** - Create `~/.openclaw/` directory
5. **Service Installation** - Optionally install as system service

### Non-Interactive Setup

For automation or CI/CD:

```bash
openclaw onboard \
  --non-interactive \
  --accept-risk \
  --flow quickstart \
  --auth-choice anthropic \
  --api-key "sk-ant-..."
```

---

## Step 2: Start the Gateway

```bash
openclaw gateway run
```

Expected output:

```
[INFO] Starting OpenClaw gateway on http://127.0.0.1:18789
[INFO] Web dashboard available at http://127.0.0.1:18789/
[INFO] Press Ctrl+C to stop
```

---

## Step 3: Open the Dashboard

Open your browser to [http://localhost:18789](http://localhost:18789)

You'll see the web dashboard with:

- **Dashboard** - System overview
- **Sessions** - Conversation history
- **Chat** - Interactive chat interface
- **Agents** - Agent configuration
- **Tools** - Available tools

---

## Step 4: Verify Everything Works

Run the health check:

```bash
openclaw doctor
```

This verifies:

- Configuration is valid
- Providers are accessible
- Gateway is running
- Workspace exists

---

## Basic Usage

### Check Status

```bash
openclaw status
```

### Interactive Chat

Use the web dashboard at `http://localhost:18789/chat` or:

```bash
# Coming soon: CLI chat
openclaw chat
```

### View Sessions

```bash
openclaw sessions list
```

---

## What's Next?

<div class="grid cards" markdown>

-   :material-cog:{ .lg .middle } **Configuration**

    ---

    Learn about the configuration file format

    [:octicons-arrow-right-24: Configuration](configuration.md)

-   :material-console:{ .lg .middle } **CLI Guide**

    ---

    Explore all CLI commands

    [:octicons-arrow-right-24: CLI Usage](../guides/cli.md)

-   :material-robot:{ .lg .middle } **AI Providers**

    ---

    Configure Anthropic, OpenAI, and more

    [:octicons-arrow-right-24: Provider Guide](../guides/providers.md)

</div>
