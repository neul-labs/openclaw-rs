# openclaw-node API

!!! info "Community Implementation"
    This is a community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw).

Node.js bindings for openclaw-rs core functionality.

---

## Installation

```bash
npm install openclaw-node
# or
yarn add openclaw-node
# or
pnpm add openclaw-node
```

---

## AnthropicProvider

### Constructor

```typescript
import { AnthropicProvider } from 'openclaw-node';

const provider = new AnthropicProvider({
  apiKey: string;              // API key (or use ANTHROPIC_API_KEY env)
  baseUrl?: string;            // Custom API endpoint
  defaultModel?: string;       // Default model to use
});
```

### Methods

#### createMessage

```typescript
interface MessageRequest {
  model: string;
  messages: Array<{
    role: 'user' | 'assistant';
    content: string | ContentBlock[];
  }>;
  maxTokens: number;
  system?: string;
  temperature?: number;
  topP?: number;
  stopSequences?: string[];
  tools?: Tool[];
}

interface MessageResponse {
  id: string;
  model: string;
  content: ContentBlock[];
  stopReason: 'end_turn' | 'max_tokens' | 'stop_sequence' | 'tool_use';
  usage: { inputTokens: number; outputTokens: number };
}

const response: MessageResponse = await provider.createMessage(request);
```

#### streamMessage

```typescript
interface StreamChunk {
  type: 'content' | 'tool_use' | 'usage' | 'done';
  text?: string;
  toolCall?: ToolCall;
  usage?: TokenUsage;
}

const stream: AsyncIterable<StreamChunk> = await provider.streamMessage(request);

for await (const chunk of stream) {
  if (chunk.type === 'content') {
    process.stdout.write(chunk.text!);
  }
}
```

---

## OpenAIProvider

### Constructor

```typescript
import { OpenAIProvider } from 'openclaw-node';

const provider = new OpenAIProvider({
  apiKey: string;              // API key (or use OPENAI_API_KEY env)
  baseUrl?: string;            // Custom API endpoint
  orgId?: string;              // Organization ID
  defaultModel?: string;       // Default model
});
```

### Methods

#### createChatCompletion

```typescript
interface ChatRequest {
  model: string;
  messages: Array<{
    role: 'system' | 'user' | 'assistant';
    content: string;
  }>;
  maxTokens?: number;
  temperature?: number;
  topP?: number;
  stop?: string[];
  functions?: Function[];
  functionCall?: 'auto' | 'none' | { name: string };
}

interface ChatResponse {
  id: string;
  model: string;
  choices: Array<{
    index: number;
    message: { role: string; content: string };
    finishReason: string;
  }>;
  usage: { promptTokens: number; completionTokens: number; totalTokens: number };
}

const response: ChatResponse = await provider.createChatCompletion(request);
```

#### streamChatCompletion

```typescript
interface ChatChunk {
  id: string;
  choices: Array<{
    index: number;
    delta: { content?: string; role?: string };
    finishReason?: string;
  }>;
}

const stream: AsyncIterable<ChatChunk> = await provider.streamChatCompletion(request);

for await (const chunk of stream) {
  const content = chunk.choices[0]?.delta?.content;
  if (content) {
    process.stdout.write(content);
  }
}
```

---

## EventBus

Pub/sub event system.

### Constructor

```typescript
import { EventBus } from 'openclaw-node';

const bus = new EventBus();
```

### Methods

#### subscribe

```typescript
type UnsubscribeFn = () => void;

const unsubscribe: UnsubscribeFn = bus.subscribe(
  eventType: string,
  callback: (event: Event) => void
);

// Later...
unsubscribe();
```

#### publish

```typescript
interface Event {
  id: string;
  type: string;
  payload: unknown;
  timestamp: number;
}

bus.publish(type: string, payload: unknown): void;
```

### Example

```typescript
const bus = new EventBus();

// Subscribe to all events
bus.subscribe('*', (event) => {
  console.log('Event:', event.type, event.payload);
});

// Subscribe to specific event
const unsub = bus.subscribe('message', (event) => {
  console.log('Message:', event.payload);
});

// Publish
bus.publish('message', { text: 'Hello!' });

// Unsubscribe
unsub();
```

---

## CredentialStore

Encrypted credential storage.

### Constructor

```typescript
import { CredentialStore } from 'openclaw-node';

const store = new CredentialStore(path?: string);
```

### Methods

```typescript
// Store a credential
await store.set(name: string, value: string): Promise<void>;

// Retrieve a credential
const value: string | null = await store.get(name: string): Promise<string | null>;

// Delete a credential
await store.delete(name: string): Promise<void>;

// Check existence
const exists: boolean = await store.exists(name: string): Promise<boolean>;

// List all credential names
const names: string[] = await store.list(): Promise<string[]>;
```

---

## AuthService

High-level authentication management.

### Constructor

```typescript
import { AuthService } from 'openclaw-node';

const auth = new AuthService();
```

### Methods

```typescript
interface ProviderAuth {
  apiKey?: string;
  apiKeyEnv?: string;
}

// Configure a provider
await auth.configureProvider(name: string, auth: ProviderAuth): Promise<void>;

// Check if configured
const configured: boolean = await auth.isProviderConfigured(name: string): Promise<boolean>;

// Get credentials
const creds: ProviderCredentials = await auth.getProviderCredentials(name: string): Promise<ProviderCredentials>;

// Clear provider config
await auth.clearProvider(name: string): Promise<void>;
```

---

## AgentRuntime

Agent with conversation management.

### Constructor

```typescript
import { AgentRuntime } from 'openclaw-node';

const agent = new AgentRuntime({
  provider: string;            // 'anthropic' | 'openai'
  model: string;               // Model identifier
  systemPrompt?: string;       // System instructions
  maxTokens?: number;          // Max response tokens
  temperature?: number;        // Sampling temperature
  tools?: string[];            // Enabled tools
});
```

### Methods

#### chat

```typescript
interface AgentResponse {
  content: string;
  toolCalls: ToolCall[];
  stopReason: string;
  usage: TokenUsage;
}

const response: AgentResponse = await agent.chat(message: string);
```

#### streamChat

```typescript
const stream: AsyncIterable<string> = agent.streamChat(message: string);

for await (const text of stream) {
  process.stdout.write(text);
}
```

#### getHistory

```typescript
const history: Message[] = agent.getHistory();
```

#### clearHistory

```typescript
agent.clearHistory(): void;
```

---

## Configuration

### loadConfig

```typescript
import { loadConfig, Config } from 'openclaw-node';

// From default path (~/.openclaw/openclaw.json)
const config: Config = await loadConfig();
```

### loadConfigFromFile

```typescript
import { loadConfigFromFile } from 'openclaw-node';

const config = await loadConfigFromFile('/path/to/config.json');
```

### Config Interface

```typescript
interface Config {
  gateway: {
    port: number;
    bind: string;
    corsOrigins: string[];
  };
  providers: Record<string, ProviderConfig>;
  agents: Record<string, AgentConfig>;
  workspace: {
    path: string;
    allowedPaths: string[];
  };
}
```

---

## Error Handling

### OpenClawError

```typescript
import { OpenClawError } from 'openclaw-node';

try {
  await provider.createMessage(request);
} catch (error) {
  if (error instanceof OpenClawError) {
    console.error('Code:', error.code);
    console.error('Message:', error.message);
  }
}
```

### Error Codes

| Code | Description |
|------|-------------|
| `INVALID_API_KEY` | Invalid or missing API key |
| `RATE_LIMITED` | Rate limit exceeded |
| `PROVIDER_ERROR` | Provider returned an error |
| `NETWORK_ERROR` | Network connectivity issue |
| `TIMEOUT` | Request timed out |
| `VALIDATION_ERROR` | Invalid request parameters |
| `CONFIG_ERROR` | Configuration issue |

---

## TypeScript Types

All types are exported:

```typescript
import type {
  // Providers
  AnthropicProvider,
  OpenAIProvider,

  // Requests
  MessageRequest,
  ChatRequest,

  // Responses
  MessageResponse,
  ChatResponse,
  StreamChunk,

  // Core types
  Message,
  Tool,
  ToolCall,
  ToolResult,

  // Events
  Event,
  EventBus,

  // Auth
  CredentialStore,
  AuthService,

  // Config
  Config,

  // Errors
  OpenClawError,
} from 'openclaw-node';
```

---

## Next Steps

[:material-console: CLI Reference](cli-commands.md){ .md-button .md-button--primary }
[:material-swap-horizontal: OpenClaw Compatibility](../compatibility/index.md){ .md-button }
