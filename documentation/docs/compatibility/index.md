# OpenClaw Compatibility

!!! info "Community Implementation"
    This is a community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw).
    We strive for maximum compatibility with the official project.

openclaw-rs is designed to be compatible with the official [OpenClaw](https://github.com/openclaw/openclaw) project.

---

## Compatibility Goals

### What We Aim For

1. **Configuration Compatibility** - Same config file format
2. **API Compatibility** - Compatible REST/WebSocket APIs
3. **Skills Compatibility** - Support for OpenClaw skills
4. **Plugin Bridge** - IPC bridge for TypeScript plugins

### What's Different

1. **Implementation Language** - Rust instead of TypeScript
2. **Performance** - Native performance, lower resource usage
3. **Some Features** - Not all features may be available yet

---

## Official OpenClaw Resources

Before diving into compatibility details, here are the official resources:

| Resource | Link |
|----------|------|
| GitHub | [github.com/openclaw/openclaw](https://github.com/openclaw/openclaw) |
| Discord | [discord.gg/openclaw](https://discord.gg/openclaw) |
| Documentation | [docs.openclaw.dev](https://docs.openclaw.dev) |

---

## Compatibility Status

| Feature | Status | Notes |
|---------|--------|-------|
| Configuration Format | ✅ Full | JSON5 config file |
| Gateway API | ✅ Full | REST and WebSocket |
| Anthropic Provider | ✅ Full | Including streaming |
| OpenAI Provider | ✅ Full | Including streaming |
| Skills System | ⚠️ Partial | Basic support |
| Plugin IPC | ⚠️ Partial | Bridge available |
| MCP Support | 🚧 Planned | In development |

Legend:
- ✅ Full support
- ⚠️ Partial support
- 🚧 Planned/In development
- ❌ Not supported

---

## Using Existing Configs

If you have an existing OpenClaw configuration, you can use it with openclaw-rs:

```bash
# Point to your existing config
openclaw --config ~/.config/openclaw/openclaw.json gateway run

# Or set environment variable
export OPENCLAW_CONFIG=~/.config/openclaw/openclaw.json
openclaw gateway run
```

See [Configuration Compatibility](config.md) for details.

---

## Migrating from OpenClaw

If you're considering using openclaw-rs alongside or instead of the official OpenClaw:

1. **Test First** - Run openclaw-rs in parallel
2. **Check Features** - Verify needed features are supported
3. **Report Issues** - Help us improve compatibility

See [Migration Guide](migration.md) for detailed steps.

---

## Contributing to Compatibility

Found a compatibility issue? Help us fix it:

1. Check if it's a [known issue](https://github.com/openclaw/openclaw-rs/issues)
2. Open an issue with reproduction steps
3. Reference the official OpenClaw behavior
4. Submit a PR if you can fix it

---

## Sections

<div class="grid cards" markdown>

-   :material-file-cog:{ .lg .middle } **Configuration**

    ---

    Config file format compatibility

    [:octicons-arrow-right-24: Configuration](config.md)

-   :material-puzzle:{ .lg .middle } **Skills System**

    ---

    OpenClaw skills support

    [:octicons-arrow-right-24: Skills](skills.md)

-   :material-power-plug:{ .lg .middle } **Plugins**

    ---

    Plugin IPC bridge

    [:octicons-arrow-right-24: Plugins](plugins.md)

-   :material-swap-horizontal:{ .lg .middle } **Migration**

    ---

    Migration considerations

    [:octicons-arrow-right-24: Migration](migration.md)

</div>
