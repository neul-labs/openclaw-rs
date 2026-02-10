# About This Project

## Overview

**openclaw-rs** is a Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw), the open-source AI agent framework. This project is developed and maintained by [Neul Labs](https://neullabs.com).

---

## Why We Built This

We built this Rust implementation to explore what the OpenClaw architecture would look like with Rust's performance and safety guarantees:

### Performance
Rust's zero-cost abstractions and lack of garbage collection enable sub-millisecond message routing and minimal memory footprint.

### Safety
Rust's ownership model prevents entire classes of bugs at compile time - no null pointer exceptions, no data races, no memory leaks.

### Portability
A single static binary can be deployed anywhere. Cross-compilation makes it easy to target different platforms including mobile and embedded.

### Interoperability
Node.js bindings via napi-rs allow seamless integration with existing JavaScript/TypeScript ecosystems.

---

## Compatibility with Original OpenClaw

We strive to maintain compatibility with the original [OpenClaw](https://github.com/openclaw/openclaw) project:

- **Configuration** - Same `~/.openclaw/openclaw.json` format (JSON5)
- **Skills** - Same Markdown + YAML frontmatter format
- **Plugins** - TypeScript plugins work via our IPC bridge
- **Session Events** - Compatible event format for session storage

This means you can potentially use openclaw-rs alongside or as an alternative to the original implementation, using the same skills and configurations.

---

## Acknowledgments

We want to express our sincere gratitude to:

- **The OpenClaw Team** - For creating such an excellent, well-designed project
- **The Open Source Community** - That makes projects like this possible
- **The Rust Ecosystem** - For providing excellent tooling and libraries

---

## Links

| Resource | Link |
|----------|------|
| Original OpenClaw | [github.com/openclaw/openclaw](https://github.com/openclaw/openclaw) |
| This Project (openclaw-rs) | [github.com/neul-labs/openclaw-rs](https://github.com/neul-labs/openclaw-rs) |
| Neul Labs | [neullabs.com](https://neullabs.com) |
| Documentation | [docs.neullabs.com/openclaw-rs](https://docs.neullabs.com/openclaw-rs/) |

---

## Contributing

We welcome contributions! Whether you're fixing bugs, adding features, or improving documentation, your help is appreciated.

See our [Contributing Guide](development/contributing.md) for details.

---

## License

This project is licensed under the MIT License.

---

## Legal Notice

- "OpenClaw" refers to the original open-source project at [github.com/openclaw/openclaw](https://github.com/openclaw/openclaw)
- "Claude" and "Anthropic" are trademarks of Anthropic, PBC
- "GPT" and "OpenAI" are trademarks of OpenAI, Inc
- All trademarks belong to their respective owners

This is an independent implementation by Neul Labs. All provider integrations use official public APIs.
