# About This Project

!!! info "Community Implementation"
    This is a community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw).

## Our Relationship to OpenClaw

[**OpenClaw**](https://github.com/openclaw/openclaw) is a popular open-source AI agent framework that enables developers to build, deploy, and manage AI-powered assistants. It's an excellent project with a vibrant community.

**This repository (openclaw-rs)** is an independent, community-driven Rust implementation. We are:

- :x: **NOT** the official OpenClaw project
- :x: **NOT** affiliated with or endorsed by the OpenClaw team
- :x: **NOT** a replacement for the official implementation

We are:

- :white_check_mark: A **tribute** to the excellent design of OpenClaw
- :white_check_mark: An **experiment** in Rust-based AI agent infrastructure
- :white_check_mark: A **learning project** for understanding AI agent architectures
- :white_check_mark: **Compatible** with OpenClaw's config, skills, and plugin ecosystem

---

## Why We Built This

We love OpenClaw and wanted to explore what a Rust implementation might offer:

### Performance
Rust's zero-cost abstractions and lack of garbage collection enable sub-millisecond message routing and minimal memory footprint.

### Safety
Rust's ownership model prevents entire classes of bugs at compile time - no null pointer exceptions, no data races, no memory leaks.

### Portability
A single static binary can be deployed anywhere. Cross-compilation makes it easy to target different platforms.

### Learning
Building this has been an incredible learning experience in understanding how AI agent systems work.

---

## What We're Grateful For

We want to express our sincere gratitude to:

- **The OpenClaw Team** - For creating such an excellent, well-designed project
- **The OpenClaw Community** - For the vibrant discussions and shared knowledge
- **The Open Source Spirit** - That makes projects like this possible

---

## Official OpenClaw Resources

Please support the official project:

| Resource | Link |
|----------|------|
| GitHub | [github.com/openclaw/openclaw](https://github.com/openclaw/openclaw) |
| Discord | [discord.gg/openclaw](https://discord.gg/openclaw) |
| Documentation | [docs.openclaw.dev](https://docs.openclaw.dev) |

---

## How We Maintain Compatibility

We strive to maintain compatibility with the official OpenClaw where it makes sense:

- **Configuration** - Same `~/.openclaw/openclaw.json` format (JSON5)
- **Skills** - Same Markdown + YAML frontmatter format
- **Plugins** - TypeScript plugins work via our IPC bridge
- **Session Events** - Compatible event format for session storage

This means you can potentially use openclaw-rs alongside or as an alternative to the official implementation, using the same skills and configurations.

---

## Contributing

We welcome contributions! Whether you're fixing bugs, adding features, or improving documentation, your help is appreciated.

See our [Contributing Guide](development/contributing.md) for details.

---

## License

This project is licensed under the MIT License - the same license as the original OpenClaw project.

---

## Legal Notice

- "Claude" and "Anthropic" are trademarks of Anthropic, PBC
- "GPT" and "OpenAI" are trademarks of OpenAI, Inc
- "OpenClaw" is the name of the original open-source project we're inspired by
- All trademarks belong to their respective owners

This is an independent community implementation. All provider integrations use official public APIs. No proprietary code from any source has been used in this project.
