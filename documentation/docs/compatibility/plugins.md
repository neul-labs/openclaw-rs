# Plugin Compatibility

!!! info "Community Implementation"
    A Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw) by [Neul Labs](https://neullabs.com).

openclaw-rs provides an IPC bridge for running OpenClaw plugins.

---

## Plugin Architecture

### OpenClaw Plugins

OpenClaw plugins are Node.js/TypeScript modules that extend functionality:

```typescript
// OpenClaw plugin
export default {
  name: 'my-plugin',
  tools: [
    {
      name: 'my_tool',
      description: 'Does something',
      execute: async (args) => {
        return { result: 'done' };
      }
    }
  ]
};
```

### openclaw-rs Approach

openclaw-rs uses an IPC bridge to run plugins in a separate Node.js process:

```
┌─────────────────┐     IPC      ┌─────────────────┐
│  openclaw-rs    │◄────────────►│  Plugin Host    │
│  (Rust)         │  JSON-RPC    │  (Node.js)      │
└─────────────────┘              └─────────────────┘
                                        │
                                        ▼
                                 ┌─────────────────┐
                                 │  Your Plugin    │
                                 │  (TypeScript)   │
                                 └─────────────────┘
```

---

## Compatibility Status

| Feature | Status | Notes |
|---------|--------|-------|
| Plugin loading | ✅ | Via IPC bridge |
| Tool registration | ✅ | Full support |
| Tool execution | ✅ | Full support |
| Event subscription | ⚠️ | Partial |
| Direct API access | ⚠️ | Limited |
| Plugin lifecycle | ✅ | Start/stop |

---

## Using Existing Plugins

### Configuration

```json5
{
  "plugins": {
    "my_plugin": {
      "path": "/path/to/plugin",
      "enabled": true,
      "config": {
        // Plugin-specific config
      }
    }
  }
}
```

### Installing the Bridge

The plugin host needs to be installed:

```bash
# Install the plugin host globally
npm install -g @openclaw-rs/plugin-host

# Or use npx
npx @openclaw-rs/plugin-host
```

### Running with Plugins

```bash
# Start gateway with plugin support
openclaw gateway run --plugins

# The plugin host starts automatically
```

---

## IPC Protocol

Communication uses JSON-RPC 2.0 over stdio:

### Register Tool

```json
{
  "jsonrpc": "2.0",
  "method": "tools.register",
  "params": {
    "name": "my_tool",
    "description": "Does something",
    "schema": {
      "type": "object",
      "properties": {
        "input": { "type": "string" }
      }
    }
  },
  "id": 1
}
```

### Execute Tool

```json
{
  "jsonrpc": "2.0",
  "method": "tools.execute",
  "params": {
    "name": "my_tool",
    "arguments": { "input": "hello" }
  },
  "id": 2
}
```

### Response

```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": "Result from tool",
    "is_error": false
  },
  "id": 2
}
```

---

## Creating Compatible Plugins

### Plugin Structure

```
my-plugin/
├── package.json
├── src/
│   └── index.ts
├── dist/
│   └── index.js
└── openclaw.json
```

### Manifest (openclaw.json)

```json
{
  "name": "my-plugin",
  "version": "1.0.0",
  "main": "dist/index.js",
  "tools": ["my_tool"],
  "permissions": ["file_read"]
}
```

### Implementation

```typescript
import { Plugin, Tool, ToolResult } from '@openclaw/plugin-sdk';

const myTool: Tool = {
  name: 'my_tool',
  description: 'Does something useful',
  schema: {
    type: 'object',
    properties: {
      input: { type: 'string', description: 'Input value' }
    },
    required: ['input']
  },
  execute: async (args: { input: string }): Promise<ToolResult> => {
    return {
      content: `Processed: ${args.input}`,
      is_error: false
    };
  }
};

const plugin: Plugin = {
  name: 'my-plugin',
  version: '1.0.0',
  tools: [myTool],

  onLoad: async () => {
    console.log('Plugin loaded');
  },

  onUnload: async () => {
    console.log('Plugin unloaded');
  }
};

export default plugin;
```

---

## Plugin Lifecycle

### Loading

1. Gateway starts
2. Plugin host spawned
3. Plugins loaded in order
4. Tools registered via IPC
5. Ready for use

### Unloading

1. Gateway shutdown signal
2. Plugins notified
3. Cleanup functions called
4. Plugin host terminates

---

## Debugging Plugins

### Enable Debug Logging

```bash
OPENCLAW_LOG=debug openclaw gateway run --plugins
```

### Plugin Host Logs

```bash
# Separate log for plugin host
tail -f ~/.openclaw/logs/plugin-host.log
```

### Manual Testing

```bash
# Run plugin host directly
npx @openclaw-rs/plugin-host --plugin /path/to/plugin
```

---

## Limitations

### Not Supported

- Direct access to Rust internals
- Synchronous tool execution
- Shared memory with gateway
- Hot reloading (restart required)

### Performance

IPC adds latency compared to native plugins:

| Operation | Native | IPC |
|-----------|--------|-----|
| Tool call | <1ms | ~5ms |
| Large data | <1ms | ~10ms |

For performance-critical plugins, consider writing a native Rust extension.

---

## Native Rust Plugins

For maximum performance, write plugins in Rust:

```rust
use openclaw_plugins::{Plugin, Tool, ToolResult};

pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str { "my-plugin" }

    fn tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(MyTool)]
    }
}

pub struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }

    async fn execute(&self, args: Value) -> Result<ToolResult, ToolError> {
        // Native performance
        Ok(ToolResult::text("Done"))
    }
}
```

---

## Next Steps

[:material-swap-horizontal: Migration Guide](migration.md){ .md-button .md-button--primary }
[:material-puzzle: Skills Compatibility](skills.md){ .md-button }
