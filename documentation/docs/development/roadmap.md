# Roadmap

!!! info "Community Implementation"
    This is a community Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw).

This roadmap outlines planned development for openclaw-rs.

---

## Current Status

### Released (v0.1.0)

- Core types and authentication
- Anthropic provider (full support)
- OpenAI provider (full support)
- Agent runtime with tool support
- Gateway with REST/WebSocket APIs
- CLI with onboarding wizard
- Vue 3 web dashboard
- Node.js bindings
- Basic skills support
- Plugin IPC bridge

---

## Near-Term Goals

### v0.2.0

**Focus: Enhanced Compatibility**

- [ ] Full skills system compatibility
- [ ] Improved plugin bridge
- [ ] MCP (Model Context Protocol) support
- [ ] Session persistence
- [ ] Better error messages

### v0.3.0

**Focus: Performance & Reliability**

- [ ] Connection pooling
- [ ] Request caching
- [ ] Automatic retry with backoff
- [ ] Health monitoring
- [ ] Metrics and observability

---

## Medium-Term Goals

### v0.4.0

**Focus: Additional Providers**

- [ ] Google Gemini provider
- [ ] Ollama provider (local models)
- [ ] Azure OpenAI provider
- [ ] Custom provider API

### v0.5.0

**Focus: Advanced Features**

- [ ] Multi-agent orchestration
- [ ] Conversation branching
- [ ] Tool result caching
- [ ] Context compression
- [ ] Rate limit management

---

## Long-Term Vision

### v1.0.0

**Focus: Production Ready**

- Stable API
- Comprehensive documentation
- Performance benchmarks
- Security audit
- Full OpenClaw compatibility

### Beyond v1.0

- Distributed mode
- Plugin marketplace
- Visual workflow builder
- Enterprise features

---

## Feature Requests

### Most Requested

Based on community feedback:

1. **MCP Support** - Model Context Protocol for tool integration
2. **Ollama Integration** - Local model support
3. **Multi-agent** - Coordinating multiple agents
4. **Memory System** - Long-term context storage
5. **Voice Support** - Audio input/output

### Compatibility Requests

1. **Full skills parity** - All OpenClaw skills working
2. **Plugin hot-reload** - Update plugins without restart
3. **Config sync** - Sync config with OpenClaw
4. **Shared sessions** - Sessions across implementations

---

## Contribution Opportunities

Areas where help is especially welcome:

### High Priority

- [ ] Writing integration tests
- [ ] Documentation improvements
- [ ] Provider implementations
- [ ] Bug fixes and error handling

### Medium Priority

- [ ] UI/UX improvements
- [ ] Performance optimization
- [ ] Cross-platform testing
- [ ] Example projects

### Good First Issues

- [ ] Documentation typos
- [ ] Error message improvements
- [ ] Test coverage
- [ ] Code cleanup

See [GitHub Issues](https://github.com/openclaw/openclaw-rs/issues) for current opportunities.

---

## Decision Log

### Architecture Decisions

| Decision | Date | Rationale |
|----------|------|-----------|
| Use Tokio runtime | 2024-01 | Best async ecosystem |
| Axum for HTTP | 2024-01 | Tower ecosystem, type-safe |
| napi-rs for Node.js | 2024-02 | Best performance |
| MkDocs for docs | 2024-03 | Material theme, search |

### Compatibility Decisions

| Decision | Date | Rationale |
|----------|------|-----------|
| JSON5 config | 2024-01 | OpenClaw compatibility |
| IPC plugin bridge | 2024-02 | Support existing plugins |
| Same port default | 2024-02 | Easy migration |

---

## Release Schedule

### Versioning

We follow [Semantic Versioning](https://semver.org/):

- **Major**: Breaking API changes
- **Minor**: New features, backwards compatible
- **Patch**: Bug fixes

### Release Cadence

- **Patch releases**: As needed for bug fixes
- **Minor releases**: Monthly target
- **Major releases**: When necessary for breaking changes

---

## How to Influence the Roadmap

### Feedback Channels

1. **GitHub Issues** - Bug reports, feature requests
2. **GitHub Discussions** - General questions, ideas
3. **Pull Requests** - Code contributions

### Voting on Features

React to GitHub issues with :thumbsup: to indicate interest. We prioritize based on community interest.

---

## Acknowledgments

This project exists because of:

- The [OpenClaw](https://github.com/openclaw/openclaw) team for inspiration
- The Rust community for excellent tools
- All contributors who help improve this project

---

## Next Steps

[:material-code-braces: Contributing Guide](contributing.md){ .md-button .md-button--primary }
[:material-hammer-wrench: Development Setup](setup.md){ .md-button }
