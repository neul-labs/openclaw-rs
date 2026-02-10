# Node.js Bindings

!!! info "Community Implementation"
    This is a community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw).

The `openclaw-node` package provides native Node.js bindings to the Rust core, offering high performance with a JavaScript-friendly API.

---

## Installation

```bash
npm install openclaw-node
# or
yarn add openclaw-node
# or
pnpm add openclaw-node
```

The package includes pre-built binaries for:

- Linux (x64, arm64)
- macOS (x64, arm64)
- Windows (x64)

---

## Quick Start

```javascript
import {
  AnthropicProvider,
  OpenAIProvider,
  EventBus
} from 'openclaw-node';

// Create a provider
const anthropic = new AnthropicProvider({
  apiKey: process.env.ANTHROPIC_API_KEY
});

// Send a message
const response = await anthropic.createMessage({
  model: 'claude-3-5-sonnet-20241022',
  maxTokens: 1024,
  messages: [
    { role: 'user', content: 'Hello, Claude!' }
  ]
});

console.log(response.content[0].text);
```

---

## Providers

### AnthropicProvider

```javascript
import { AnthropicProvider } from 'openclaw-node';

const provider = new AnthropicProvider({
  apiKey: process.env.ANTHROPIC_API_KEY,
  // Optional:
  baseUrl: 'https://api.anthropic.com',
  defaultModel: 'claude-3-5-sonnet-20241022'
});
```

#### Methods

**createMessage(request)**

Send a message and get a complete response.

```javascript
const response = await provider.createMessage({
  model: 'claude-3-5-sonnet-20241022',
  maxTokens: 4096,
  messages: [
    { role: 'user', content: 'Explain async/await' }
  ],
  system: 'You are a helpful programming tutor.'
});
```

**streamMessage(request)**

Stream a response chunk by chunk.

```javascript
const stream = await provider.streamMessage({
  model: 'claude-3-5-sonnet-20241022',
  maxTokens: 4096,
  messages: [
    { role: 'user', content: 'Write a story' }
  ]
});

for await (const chunk of stream) {
  if (chunk.type === 'content') {
    process.stdout.write(chunk.text);
  }
}
```

### OpenAIProvider

```javascript
import { OpenAIProvider } from 'openclaw-node';

const provider = new OpenAIProvider({
  apiKey: process.env.OPENAI_API_KEY,
  // Optional:
  baseUrl: 'https://api.openai.com/v1',
  orgId: 'org-...',
  defaultModel: 'gpt-4o'
});
```

#### Methods

**createChatCompletion(request)**

```javascript
const response = await provider.createChatCompletion({
  model: 'gpt-4o',
  messages: [
    { role: 'system', content: 'You are helpful.' },
    { role: 'user', content: 'Hello!' }
  ],
  maxTokens: 1024
});
```

**streamChatCompletion(request)**

```javascript
const stream = await provider.streamChatCompletion({
  model: 'gpt-4o',
  messages: [
    { role: 'user', content: 'Tell me a joke' }
  ]
});

for await (const chunk of stream) {
  if (chunk.content) {
    process.stdout.write(chunk.content);
  }
}
```

---

## Event System

The EventBus provides pub/sub messaging between components.

```javascript
import { EventBus } from 'openclaw-node';

// Create an event bus
const bus = new EventBus();

// Subscribe to events
const unsubscribe = bus.subscribe('message', (event) => {
  console.log('Received:', event.payload);
});

// Publish events
bus.publish('message', { text: 'Hello!' });

// Unsubscribe
unsubscribe();
```

### Event Types

```typescript
interface Event {
  id: string;
  type: string;
  payload: unknown;
  timestamp: number;
}
```

---

## Agent Runtime

```javascript
import { AgentRuntime } from 'openclaw-node';

const agent = new AgentRuntime({
  provider: 'anthropic',
  model: 'claude-3-5-sonnet-20241022',
  systemPrompt: 'You are a helpful assistant.',
  tools: ['file_read', 'bash']
});

// Chat with the agent
const response = await agent.chat('What files are in the current directory?');
console.log(response.content);

// Stream responses
for await (const chunk of agent.streamChat('Explain this codebase')) {
  process.stdout.write(chunk);
}
```

---

## Authentication

### CredentialStore

Securely manage API credentials.

```javascript
import { CredentialStore } from 'openclaw-node';

const store = new CredentialStore('~/.openclaw/credentials');

// Store a credential
await store.set('anthropic', 'sk-ant-...');

// Retrieve a credential
const apiKey = await store.get('anthropic');

// Delete a credential
await store.delete('anthropic');
```

### AuthService

Higher-level authentication management.

```javascript
import { AuthService } from 'openclaw-node';

const auth = new AuthService();

// Configure a provider
await auth.configureProvider('anthropic', {
  apiKey: process.env.ANTHROPIC_API_KEY
});

// Check if configured
const isConfigured = await auth.isProviderConfigured('anthropic');

// Get provider credentials
const creds = await auth.getProviderCredentials('anthropic');
```

---

## TypeScript Support

Full TypeScript definitions are included:

```typescript
import {
  AnthropicProvider,
  MessageRequest,
  MessageResponse,
  StreamChunk
} from 'openclaw-node';

const provider = new AnthropicProvider({
  apiKey: process.env.ANTHROPIC_API_KEY!
});

const request: MessageRequest = {
  model: 'claude-3-5-sonnet-20241022',
  maxTokens: 1024,
  messages: [
    { role: 'user', content: 'Hello!' }
  ]
};

const response: MessageResponse = await provider.createMessage(request);
```

---

## Error Handling

```javascript
import { AnthropicProvider, OpenClawError } from 'openclaw-node';

try {
  const response = await provider.createMessage(request);
} catch (error) {
  if (error instanceof OpenClawError) {
    console.error('OpenClaw error:', error.code, error.message);
  } else {
    throw error;
  }
}
```

### Error Codes

| Code | Description |
|------|-------------|
| `INVALID_API_KEY` | API key is invalid |
| `RATE_LIMITED` | Too many requests |
| `PROVIDER_ERROR` | Provider returned an error |
| `NETWORK_ERROR` | Network connectivity issue |
| `TIMEOUT` | Request timed out |

---

## Configuration

### From Environment

```javascript
import { loadConfig } from 'openclaw-node';

// Load from OPENCLAW_CONFIG or default path
const config = await loadConfig();
```

### From File

```javascript
import { loadConfigFromFile } from 'openclaw-node';

const config = await loadConfigFromFile('~/.openclaw/openclaw.json');
```

---

## Performance

The Node.js bindings use napi-rs for native performance:

- **Zero-copy** where possible
- **Async by default** using Tokio runtime
- **Stream support** for memory-efficient processing

### Benchmarks

| Operation | openclaw-node | Pure JS |
|-----------|---------------|---------|
| Message (simple) | ~50ms | ~200ms |
| Streaming | Native perf | N/A |
| Auth encryption | <1ms | ~10ms |

---

## Examples

### Chat Bot

```javascript
import { AnthropicProvider } from 'openclaw-node';
import readline from 'readline';

const provider = new AnthropicProvider({
  apiKey: process.env.ANTHROPIC_API_KEY
});

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout
});

const messages = [];

async function chat(userMessage) {
  messages.push({ role: 'user', content: userMessage });

  const response = await provider.createMessage({
    model: 'claude-3-5-sonnet-20241022',
    maxTokens: 1024,
    messages
  });

  const assistantMessage = response.content[0].text;
  messages.push({ role: 'assistant', content: assistantMessage });

  return assistantMessage;
}

rl.on('line', async (line) => {
  const response = await chat(line);
  console.log('\nAssistant:', response, '\n');
});
```

### Tool Use

```javascript
import { AgentRuntime } from 'openclaw-node';

const agent = new AgentRuntime({
  provider: 'anthropic',
  model: 'claude-3-5-sonnet-20241022',
  tools: ['bash', 'file_read']
});

// Agent will use tools as needed
const response = await agent.chat(
  'List all TypeScript files in the current directory'
);
```

---

## Next Steps

[:material-monitor-dashboard: Web Dashboard](web-dashboard.md){ .md-button .md-button--primary }
[:material-api: Full API Reference](../reference/node.md){ .md-button }
