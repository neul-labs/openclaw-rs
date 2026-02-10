# openclaw-core

Core types, configuration, events, and security primitives for [OpenClaw](https://github.com/openclaw/openclaw-rs).

## Features

- **Types**: Core identifiers (AgentId, SessionKey, ChannelId, PeerId)
- **Config**: JSON5 configuration loading and validation
- **Events**: Append-only event store with CRDT projections (sled-backed)
- **Secrets**: AES-256-GCM encrypted credential storage
- **Auth**: Authentication profile management
- **Validation**: Input validation and sanitization

## Usage

```rust
use openclaw_core::{Config, EventStore, SessionEvent, SessionEventKind, ApiKey};

// Load configuration
let config = Config::load()?;

// Create event store
let store = EventStore::open(Path::new("~/.openclaw/sessions"))?;

// Append event
let event = SessionEvent::new(
    session_key,
    "default".to_string(),
    SessionEventKind::MessageReceived {
        content: "Hello".to_string(),
        attachments: vec![],
    },
);
store.append(&event)?;

// Get projection
let projection = store.get_projection(&session_key)?;

// Secure API key handling
let key = ApiKey::new("sk-...".to_string());
println!("{}", key); // Prints "[REDACTED]"
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.
