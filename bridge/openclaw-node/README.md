# openclaw-node

> **Node.js bindings for the Rust implementation of OpenClaw by [Neul Labs](https://neullabs.com)**

[![Crates.io](https://img.shields.io/crates/v/openclaw-node.svg)](https://crates.io/crates/openclaw-node)
[![npm](https://img.shields.io/npm/v/openclaw-node.svg)](https://www.npmjs.com/package/openclaw-node)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

Part of [openclaw-rs](https://github.com/neul-labs/openclaw-rs), a Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw) by [Neul Labs](https://neullabs.com). This package provides napi-rs bindings exposing the Rust core to Node.js applications.

## Features

- **AI Providers**: Anthropic Claude and OpenAI GPT clients with streaming support
- **Authentication**: Safe API key handling with encrypted credential storage (AES-256-GCM)
- **Event Store**: Append-only event storage with CRDT projections
- **Tool Registry**: Register and execute tools from JavaScript
- **Configuration**: Load and validate OpenClaw config files
- **Validation**: Input validation and session key building

## Installation

```bash
# From npm (when published)
npm install openclaw-node

# From source
cd bridge/openclaw-node
npm run build
```

## Quick Start

```javascript
const {
  AnthropicProvider,
  OpenAIProvider,
  NodeApiKey,
  CredentialStore,
  NodeEventStore,
  loadDefaultConfig,
} = require('openclaw-node');

// Create an Anthropic provider
const anthropic = new AnthropicProvider(process.env.ANTHROPIC_API_KEY);

// Create a completion
const response = await anthropic.complete({
  model: 'claude-3-5-sonnet-20241022',
  messages: [{ role: 'user', content: 'Hello!' }],
  maxTokens: 1024,
});

console.log(response.content);
```

## API Reference

### Providers

#### AnthropicProvider

```javascript
// Create with API key
const provider = new AnthropicProvider('sk-ant-...');

// Or with custom base URL (for proxies)
const provider = AnthropicProvider.withBaseUrl('sk-ant-...', 'https://proxy.example.com');

// Get provider name
console.log(provider.name); // "anthropic"

// List available models
const models = await provider.listModels();

// Create completion
const response = await provider.complete({
  model: 'claude-3-5-sonnet-20241022',
  messages: [
    { role: 'user', content: 'What is 2+2?' }
  ],
  maxTokens: 100,
  temperature: 0.7,
  system: 'You are a helpful assistant.',
});

console.log(response.content);
console.log(response.usage); // { inputTokens, outputTokens, ... }
```

#### OpenAIProvider

```javascript
// Create with API key
const provider = new OpenAIProvider('sk-...');

// With organization ID
const provider = OpenAIProvider.withOrg('sk-...', 'org-...');

// With custom base URL (for Azure, LocalAI, etc.)
const provider = OpenAIProvider.withBaseUrl('sk-...', 'https://api.openai.azure.com');

// Same API as AnthropicProvider
const response = await provider.complete({
  model: 'gpt-4o',
  messages: [{ role: 'user', content: 'Hello!' }],
  maxTokens: 100,
});
```

#### Streaming

```javascript
provider.completeStream(
  {
    model: 'claude-3-5-sonnet-20241022',
    messages: [{ role: 'user', content: 'Write a short story.' }],
    maxTokens: 500,
  },
  (err, chunk) => {
    if (err) {
      console.error('Error:', err);
      return;
    }

    if (chunk.chunkType === 'content_block_delta' && chunk.delta) {
      process.stdout.write(chunk.delta);
    }

    if (chunk.chunkType === 'message_stop') {
      console.log('\nDone!');
    }
  }
);
```

### Request/Response Types

```typescript
interface JsMessage {
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string;
  toolUseId?: string;   // For tool results
  toolName?: string;    // For tool results
}

interface JsCompletionRequest {
  model: string;
  messages: JsMessage[];
  system?: string;
  maxTokens: number;
  temperature?: number;
  stop?: string[];
  tools?: JsTool[];
}

interface JsCompletionResponse {
  id: string;
  model: string;
  content: string;
  stopReason?: 'end_turn' | 'max_tokens' | 'stop_sequence' | 'tool_use';
  toolCalls?: JsToolCall[];
  usage: JsTokenUsage;
}

interface JsStreamChunk {
  chunkType: 'message_start' | 'content_block_start' | 'content_block_delta'
           | 'content_block_stop' | 'message_delta' | 'message_stop';
  delta?: string;
  index?: number;
  stopReason?: string;
}
```

### Authentication

#### NodeApiKey

Safe API key wrapper that prevents accidental logging.

```javascript
const key = new NodeApiKey('sk-secret-key');

// Safe for logging - shows "[REDACTED]"
console.log(key.toString()); // "[REDACTED]"

// Get actual value when needed for API calls
const secret = key.exposeSecretForApiCall();

// Check properties without exposing
console.log(key.isEmpty());           // false
console.log(key.length());            // 13
console.log(key.startsWith('sk-'));   // true
```

#### CredentialStore

Encrypted credential storage using AES-256-GCM.

```javascript
const crypto = require('crypto');

// Generate a 32-byte encryption key (store this securely!)
const encryptionKey = crypto.randomBytes(32).toString('hex');

// Create store
const store = new CredentialStore(encryptionKey, './credentials');

// Store a credential
const apiKey = new NodeApiKey('sk-ant-...');
await store.store('anthropic-main', apiKey);

// Load a credential
const loaded = await store.load('anthropic-main');
console.log(loaded.exposeSecretForApiCall()); // 'sk-ant-...'

// List stored credentials
const names = await store.list(); // ['anthropic-main']

// Delete a credential
await store.delete('anthropic-main');
```

### Tool Registry

Register and execute tools from JavaScript.

```javascript
const { ToolRegistry } = require('openclaw-node');

const registry = new ToolRegistry();

// Register a tool
registry.register({
  name: 'get_weather',
  description: 'Get the current weather for a location',
  inputSchema: {
    type: 'object',
    properties: {
      location: { type: 'string', description: 'City name' }
    },
    required: ['location']
  }
});

// List registered tools
console.log(registry.list()); // ['get_weather']

// Execute a tool
const result = await registry.execute('get_weather', { location: 'London' });
console.log(result); // { success: true, result: ... }
```

### Event Store

Append-only event storage with CRDT projections.

```javascript
const { NodeEventStore, buildSessionKey } = require('openclaw-node');

// Create event store
const store = new NodeEventStore('./events');

// Build a session key
const sessionKey = buildSessionKey(
  'default',     // agentId
  'telegram',    // channel
  'bot123',      // accountId
  'user',        // peerType
  'user456'      // peerId
);

// Append events
store.appendEvent(
  sessionKey,
  'default',
  'message_received',
  JSON.stringify({ content: 'Hello!' })
);

store.appendEvent(
  sessionKey,
  'default',
  'agent_response',
  JSON.stringify({
    content: 'Hi there!',
    model: 'claude-3-5-sonnet',
    tokens: { input_tokens: 10, output_tokens: 5 }
  })
);

// Get all events for a session
const events = JSON.parse(store.getEvents(sessionKey));

// Get materialized projection
const projection = JSON.parse(store.getProjection(sessionKey));

// List all sessions
const sessions = store.listSessions();

// Flush to disk
store.flush();
```

### Configuration

```javascript
const { loadConfig, loadDefaultConfig, validateConfig } = require('openclaw-node');

// Load config from path
const config = JSON.parse(loadConfig('/path/to/openclaw.json'));

// Load from default location (~/.openclaw/openclaw.json)
const defaultConfig = JSON.parse(loadDefaultConfig());

// Validate a config file
const validation = JSON.parse(validateConfig('/path/to/openclaw.json'));
if (validation.valid) {
  console.log('Config is valid');
} else {
  console.log('Errors:', validation.errors);
}
```

### Validation

```javascript
const { validateMessage, validatePath } = require('openclaw-node');

// Validate message content
const msgResult = JSON.parse(validateMessage('Hello!', 10000));
if (msgResult.valid) {
  console.log('Message is valid');
}

// Validate file path (prevents traversal attacks)
const pathResult = JSON.parse(validatePath('/safe/path/file.txt'));
if (pathResult.valid) {
  console.log('Path is valid');
}
```

## Error Handling

Errors are returned as JSON with structured information:

```javascript
try {
  await provider.complete(request);
} catch (error) {
  const err = JSON.parse(error.message);
  console.log(err.code);        // 'RATE_LIMITED', 'PROVIDER_API_ERROR', etc.
  console.log(err.message);     // Human-readable message
  console.log(err.status);      // HTTP status code (if applicable)
  console.log(err.retryAfter);  // Seconds to wait (for rate limits)
}
```

Error codes:
- `PROVIDER_API_ERROR` - API returned an error
- `RATE_LIMITED` - Rate limit exceeded (check `retryAfter`)
- `NETWORK_ERROR` - Network connectivity issue
- `CONFIG_ERROR` - Configuration error
- `CREDENTIAL_NOT_FOUND` - Credential not in store
- `CRYPTO_ERROR` - Encryption/decryption failed

## Building from Source

```bash
# Prerequisites
# - Rust 1.85+
# - Node.js 20+

# Clone the repository
git clone https://github.com/neul-labs/openclaw-rs
cd openclaw-rs/bridge/openclaw-node

# Build with napi
npm install
npm run build

# Run tests
cargo test
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.
