# Agent Runtime Guide

!!! info "Community Implementation"
    This is a community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw).

Agents are the core abstraction for AI interactions in openclaw-rs. They manage conversations, tool execution, and provider communication.

---

## What is an Agent?

An agent is a configured AI assistant with:

- **Provider** - Which AI service to use (Anthropic, OpenAI)
- **Model** - Which model to use
- **System Prompt** - Initial instructions
- **Tools** - Available capabilities
- **Parameters** - Temperature, max tokens, etc.

---

## Configuration

### Basic Agent

```json5
{
  "agents": {
    "default": {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022",
      "system_prompt": "You are a helpful assistant.",
      "max_tokens": 4096
    }
  }
}
```

### Full Configuration

```json5
{
  "agents": {
    "code-assistant": {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022",
      "system_prompt": "You are an expert programmer. Help with code questions.",
      "max_tokens": 8192,
      "temperature": 0.7,
      "tools": ["file_read", "file_write", "bash"],
      "stop_sequences": ["```"]
    }
  }
}
```

### Configuration Options

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `provider` | string | required | Provider name |
| `model` | string | required | Model identifier |
| `system_prompt` | string | `""` | System instructions |
| `max_tokens` | number | `4096` | Max response tokens |
| `temperature` | number | `1.0` | Sampling temperature (0-1) |
| `tools` | string[] | `[]` | Enabled tool names |
| `stop_sequences` | string[] | `[]` | Stop generation sequences |

---

## Multiple Agents

Define specialized agents for different tasks:

```json5
{
  "agents": {
    "default": {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022",
      "system_prompt": "You are a helpful assistant."
    },
    "coder": {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022",
      "system_prompt": "You are an expert programmer.",
      "tools": ["file_read", "file_write", "bash"],
      "temperature": 0.3
    },
    "creative": {
      "provider": "openai",
      "model": "gpt-4o",
      "system_prompt": "You are a creative writer.",
      "temperature": 0.9
    }
  }
}
```

---

## Agent Runtime

### Lifecycle

1. **Initialize** - Load configuration, connect to provider
2. **Receive Message** - User sends a message
3. **Process** - Send to AI, handle tool calls
4. **Respond** - Stream or return response
5. **Repeat** - Continue conversation

### Sessions

Each conversation is tracked in a session:

```bash
# List sessions
openclaw sessions list

# View session
openclaw sessions show <SESSION_ID>
```

---

## Tool Integration

Agents can use tools to interact with the system.

### Built-in Tools

| Tool | Description |
|------|-------------|
| `file_read` | Read file contents |
| `file_write` | Write to files |
| `bash` | Execute shell commands |
| `web_fetch` | Fetch URLs |

### Enabling Tools

```json5
{
  "agents": {
    "default": {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022",
      "tools": ["file_read", "file_write", "bash"]
    }
  }
}
```

### Tool Execution Flow

1. AI requests tool use
2. Gateway validates request
3. Tool executes in sandbox
4. Result returned to AI
5. AI continues with result

---

## Streaming Responses

The agent runtime supports streaming for real-time responses:

```
User: Explain quantum computing

Agent (streaming):
Q|ua|nt|um| c|om|pu|ti|ng| i|s...
```

### Via WebSocket

Connect to the WebSocket endpoint for streaming:

```javascript
const ws = new WebSocket('ws://localhost:18789/ws');

ws.send(JSON.stringify({
  jsonrpc: '2.0',
  method: 'chat.stream',
  params: { message: 'Hello!' },
  id: 1
}));

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  if (data.params?.chunk) {
    process.stdout.write(data.params.chunk);
  }
};
```

---

## Context Management

### Token Limits

The runtime tracks token usage:

- Input tokens (prompt)
- Output tokens (response)
- Context window usage

### Conversation History

History is automatically managed:

```json5
{
  "agents": {
    "default": {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022",
      "context": {
        "max_history_messages": 50,
        "summarize_after": 30
      }
    }
  }
}
```

---

## Error Handling

### Graceful Degradation

The runtime handles:

- Provider timeouts
- Rate limiting
- Tool failures
- Token limit exceeded

### Retry Logic

```json5
{
  "agents": {
    "default": {
      "retry": {
        "max_attempts": 3,
        "backoff_multiplier": 2
      }
    }
  }
}
```

---

## Monitoring

### Metrics

The gateway exposes agent metrics:

- Request count
- Response times
- Token usage
- Error rates

### Logs

```bash
# View agent logs
OPENCLAW_LOG=debug openclaw gateway run
```

---

## Programmatic Usage

### Rust

```rust
use openclaw_agents::{AgentRuntime, AgentConfig};

let config = AgentConfig::builder()
    .provider("anthropic")
    .model("claude-3-5-sonnet-20241022")
    .system_prompt("You are helpful.")
    .build()?;

let runtime = AgentRuntime::new(config)?;
let response = runtime.chat("Hello!").await?;
```

### Node.js

```javascript
import { AgentRuntime } from 'openclaw-node';

const agent = new AgentRuntime({
  provider: 'anthropic',
  model: 'claude-3-5-sonnet-20241022'
});

const response = await agent.chat('Hello!');
```

---

## Next Steps

[:material-nodejs: Node.js Bindings](nodejs.md){ .md-button .md-button--primary }
[:material-tools: Tool Reference](../reference/agents.md){ .md-button }
