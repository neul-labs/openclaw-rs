# TypeScript Interoperability

OpenClaw Rust Core is designed to integrate seamlessly with the existing TypeScript ecosystem, enabling gradual migration without breaking changes.

## Strategy

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                          Interoperability Architecture                           │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   TypeScript Ecosystem                      Rust Core                            │
│   ───────────────────                       ─────────                            │
│                                                                                  │
│   ┌─────────────────┐                      ┌─────────────────┐                  │
│   │   Skills        │ ───── (file) ─────▶  │   Skill Loader  │                  │
│   │   (Markdown)    │                      │                 │                  │
│   └─────────────────┘                      └─────────────────┘                  │
│                                                                                  │
│   ┌─────────────────┐                      ┌─────────────────┐                  │
│   │   Plugins       │ ───── (IPC) ──────▶  │   Plugin Bridge │                  │
│   │   (TypeScript)  │                      │                 │                  │
│   └─────────────────┘                      └─────────────────┘                  │
│                                                                                  │
│   ┌─────────────────┐                      ┌─────────────────┐                  │
│   │   Node.js App   │ ───── (napi) ─────▶  │   napi Bindings │                  │
│   │                 │                      │                 │                  │
│   └─────────────────┘                      └─────────────────┘                  │
│                                                                                  │
│   ┌─────────────────┐                      ┌─────────────────┐                  │
│   │   Config        │ ───── (file) ─────▶  │   Config Loader │                  │
│   │   (JSON5)       │                      │                 │                  │
│   └─────────────────┘                      └─────────────────┘                  │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## Skills System

Skills continue to use the same Markdown + YAML frontmatter format:

```markdown
---
name: my-skill
description: A custom skill
tools:
  - bash
  - read_file
---

# My Skill

Instructions for the AI...
```

The Rust core loads skills from the same locations:
- `~/.openclaw/skills/`
- `./skills/` (project-local)

## Plugin SDK Compatibility

TypeScript plugins built with `openclaw/plugin-sdk` work via the IPC bridge:

```typescript
// TypeScript plugin (unchanged)
import { definePlugin } from 'openclaw/plugin-sdk';

export default definePlugin({
  name: 'my-plugin',
  hooks: {
    onMessage: async (message, context) => {
      // Plugin logic
    }
  }
});
```

### IPC Protocol

```
┌────────────────┐                    ┌────────────────┐
│  Rust Core     │                    │  TS Plugin     │
│                │                    │                │
│  ┌──────────┐  │    IPC (nng)       │  ┌──────────┐  │
│  │  Plugin  │  │ ◀──────────────▶   │  │  Plugin  │  │
│  │  Bridge  │  │    JSON-RPC        │  │  Runtime │  │
│  └──────────┘  │                    │  └──────────┘  │
└────────────────┘                    └────────────────┘
```

Messages:

```json
// Request (Rust → TS)
{
  "jsonrpc": "2.0",
  "method": "onMessage",
  "params": {
    "message": { "content": "Hello" },
    "context": { "sessionKey": "..." }
  },
  "id": 1
}

// Response (TS → Rust)
{
  "jsonrpc": "2.0",
  "result": { "handled": true },
  "id": 1
}
```

## napi-rs Bindings

For Node.js applications, the Rust core exposes bindings via napi-rs:

```typescript
// Using Rust core from Node.js
import { OpenClawCore } from 'openclaw-node';

const core = new OpenClawCore({
  configPath: '~/.openclaw/openclaw.json'
});

// Use Rust event store
const events = await core.getSessionEvents(sessionKey);

// Use Rust validation
const result = core.validateMessage(content);
```

### Exposed APIs

| Module | Functions |
|--------|-----------|
| Config | `loadConfig()`, `validateConfig()` |
| Events | `appendEvent()`, `getEvents()`, `getProjection()` |
| Secrets | `storeCredential()`, `loadCredential()` |
| Validation | `validateMessage()`, `sanitizeContent()` |

## Configuration Compatibility

The Rust core reads the same `~/.openclaw/openclaw.json` format:

```json5
{
  "agents": {
    "default": {
      "model": "claude-sonnet-4-20250514",
      "provider": "anthropic"
    }
  },
  "channels": {
    "telegram": {
      "enabled": true,
      "botToken": "env:TELEGRAM_BOT_TOKEN"
    }
  },
  "gateway": {
    "port": 18789,
    "mode": "local"
  }
}
```

Environment variable expansion (`env:VAR_NAME`) works identically.

## Session Storage Migration

Existing sessions can be migrated to the Rust event store:

```
┌─────────────────────────────────────────────────────────────────┐
│                     Migration Process                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. Read existing TS sessions                                    │
│     ~/.openclaw/sessions/*.json                                  │
│                                                                  │
│  2. Convert to event format                                      │
│     Message → SessionEventKind::MessageReceived                  │
│     Response → SessionEventKind::AgentResponse                   │
│                                                                  │
│  3. Write to Rust event store                                    │
│     EventStore::append()                                         │
│                                                                  │
│  4. Verify projection matches                                    │
│     Compare message counts, timestamps                           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Gradual Migration Path

### Phase 1: Parallel Operation

Both TypeScript and Rust components run simultaneously:

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                  │
│  ┌─────────────────┐         ┌─────────────────┐                │
│  │  TS Gateway     │ ◀─────▶ │  Rust Core      │                │
│  │  (primary)      │   IPC   │  (library)      │                │
│  └─────────────────┘         └─────────────────┘                │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Phase 2: Rust Primary

Rust gateway becomes primary, TS plugins via IPC:

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                  │
│  ┌─────────────────┐         ┌─────────────────┐                │
│  │  Rust Gateway   │ ◀─────▶ │  TS Plugins     │                │
│  │  (primary)      │   IPC   │  (extensions)   │                │
│  └─────────────────┘         └─────────────────┘                │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Phase 3: Full Rust (Optional)

Complete Rust implementation for performance-critical deployments:

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    Rust Core                                 ││
│  │  Gateway + Agents + Channels + Plugins (WASM)               ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## API Compatibility Matrix

| Feature | TypeScript | Rust | Notes |
|---------|------------|------|-------|
| Config loading | ✅ | ✅ | Same format |
| Session storage | JSON files | sled (events) | Migration tool |
| Skills | ✅ | ✅ | Same format |
| Plugins (TS) | Native | IPC bridge | Transparent |
| Plugins (WASM) | ❌ | 🔜 | Rust-only |
| Provider APIs | ✅ | ⚠️ Partial | In progress |
| Channel adapters | ✅ | 🔜 | Planned |

## Testing Interop

```bash
# Run TypeScript tests against Rust core
pnpm test:interop

# Verify config compatibility
cargo run -p openclaw-cli -- config validate

# Test IPC bridge
cargo run -p openclaw-cli -- plugin test my-plugin
```

## Breaking Changes

The Rust core aims for zero breaking changes, but some differences exist:

| Area | TypeScript | Rust | Migration |
|------|------------|------|-----------|
| Session format | JSON files | Event log | Migration tool |
| Error codes | Strings | Typed enums | Mapping layer |
| Timestamps | ISO strings | chrono DateTime | Auto-convert |

## Resources

- [OpenClaw TypeScript](https://github.com/openclaw/openclaw)
- [Plugin SDK](https://github.com/openclaw/openclaw/tree/main/packages/plugin-sdk)
- [napi-rs](https://napi.rs/)
