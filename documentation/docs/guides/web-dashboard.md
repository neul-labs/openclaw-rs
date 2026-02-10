# Web Dashboard

!!! info "Community Implementation"
    This is a community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw).

The web dashboard provides a browser-based interface for interacting with openclaw-rs. Built with Vue 3 and Pinia, it's embedded directly in the gateway.

---

## Accessing the Dashboard

Start the gateway and open your browser:

```bash
openclaw gateway run
# Dashboard available at http://localhost:18789
```

The dashboard is served directly from the gateway - no separate process needed.

---

## Features

### Dashboard Overview

The main dashboard shows:

- **System Status** - Gateway health, uptime
- **Active Sessions** - Current conversations
- **Provider Status** - API connectivity
- **Recent Activity** - Latest interactions

### Chat Interface

Interactive chat with AI agents:

- Real-time streaming responses
- Markdown rendering
- Code syntax highlighting
- Conversation history
- Multiple agent selection

### Session Management

View and manage conversation sessions:

- List all sessions
- Search and filter
- View full history
- Delete sessions
- Export conversations

### Agent Configuration

Configure agents through the UI:

- Create new agents
- Edit system prompts
- Select models
- Configure tools
- Adjust parameters

### Tool Viewer

Browse available tools:

- Tool descriptions
- Parameter schemas
- Usage examples
- Enable/disable tools

---

## Navigation

| Section | Path | Description |
|---------|------|-------------|
| Dashboard | `/` | System overview |
| Chat | `/chat` | Interactive chat |
| Sessions | `/sessions` | Session management |
| Agents | `/agents` | Agent configuration |
| Tools | `/tools` | Available tools |
| Settings | `/settings` | System settings |

---

## Chat Interface

### Starting a Conversation

1. Navigate to `/chat`
2. Select an agent from the dropdown
3. Type your message
4. Press Enter or click Send

### Streaming Responses

Responses stream in real-time via WebSocket. You'll see text appear character by character as the AI generates its response.

### Markdown Support

The chat interface renders:

- **Bold** and *italic* text
- Code blocks with syntax highlighting
- Lists and tables
- Links and images

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Enter` | Send message |
| `Shift+Enter` | New line |
| `Ctrl+K` | Clear chat |
| `Ctrl+/` | Toggle sidebar |

---

## Session Management

### Viewing Sessions

1. Navigate to `/sessions`
2. Browse the session list
3. Click a session to view details

### Session Details

Each session shows:

- **Created** - When the conversation started
- **Last Active** - Most recent message
- **Agent** - Which agent was used
- **Messages** - Full conversation history
- **Tokens** - Total token usage

### Exporting

Export conversations in multiple formats:

- **JSON** - Full structured data
- **Markdown** - Readable document
- **Text** - Plain text

---

## Agent Configuration

### Creating an Agent

1. Navigate to `/agents`
2. Click "New Agent"
3. Configure:
   - Name
   - Provider
   - Model
   - System prompt
   - Tools
4. Click Save

### Editing Agents

1. Click an agent in the list
2. Modify settings
3. Click Save

Changes take effect for new sessions.

---

## Settings

### Gateway Settings

- **Port** - Server port
- **Bind Address** - Network interface
- **CORS Origins** - Allowed origins

### Provider Settings

- **API Keys** - Manage credentials
- **Default Models** - Set defaults

### UI Settings

- **Theme** - Light/Dark mode
- **Font Size** - Text size
- **Animations** - Enable/disable

---

## Building from Source

The UI is built with Vue 3 and embedded in the gateway:

```bash
cd crates/openclaw-ui
npm install
npm run build
```

### Development Mode

For UI development with hot reload:

```bash
# Terminal 1: Start gateway
openclaw gateway run

# Terminal 2: Start dev server
cd crates/openclaw-ui
npm run dev
```

The dev server proxies API requests to the gateway.

---

## Technology Stack

| Component | Technology |
|-----------|------------|
| Framework | Vue 3 |
| State | Pinia |
| Router | Vue Router |
| Build | Vite |
| Styling | Tailwind CSS |
| Icons | Heroicons |

---

## API Integration

The dashboard communicates with the gateway via:

### REST API

```javascript
// Example: List sessions
fetch('/api/sessions')
  .then(res => res.json())
  .then(sessions => console.log(sessions));
```

### WebSocket

```javascript
// Example: Stream chat
const ws = new WebSocket('ws://localhost:18789/ws');
ws.send(JSON.stringify({
  jsonrpc: '2.0',
  method: 'chat.stream',
  params: { message: 'Hello!' },
  id: 1
}));
```

---

## Customization

### Theming

The UI uses CSS variables for theming:

```css
:root {
  --primary-color: #3f51b5;
  --background: #ffffff;
  --text: #1a1a1a;
}

[data-theme="dark"] {
  --background: #1a1a2e;
  --text: #ffffff;
}
```

### Embedding

The dashboard can be embedded in other applications:

```html
<iframe src="http://localhost:18789/chat" />
```

---

## Troubleshooting

### Dashboard Not Loading

1. Check gateway is running: `openclaw status`
2. Verify port is correct
3. Check browser console for errors

### WebSocket Disconnects

- Check network stability
- Verify CORS settings
- Look for proxy issues

### Slow Performance

- Reduce conversation history
- Check system resources
- Disable animations

---

## Next Steps

[:material-cog: Architecture Overview](../architecture/index.md){ .md-button .md-button--primary }
[:material-api: API Reference](../reference/core.md){ .md-button }