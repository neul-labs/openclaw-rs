# Contributing

Thank you for your interest in contributing to OpenClaw Rust Core!

## Development Setup

### Prerequisites

- Rust 1.85+ (2024 edition)
- Cargo
- System dependencies:
  - Linux: `bubblewrap` for sandboxing tests
  - macOS: Xcode Command Line Tools

### Getting Started

```bash
# Clone the repository
git clone https://github.com/openclaw/openclaw-rs
cd openclaw-rs

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run clippy
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all
```

## Project Structure

```
openclaw-rs/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── openclaw-core/      # Foundation types, config, events
│   ├── openclaw-ipc/       # IPC message types
│   ├── openclaw-providers/ # AI provider clients
│   ├── openclaw-agents/    # Agent runtime, sandbox
│   ├── openclaw-channels/  # Channel adapters
│   ├── openclaw-gateway/   # HTTP/WS server
│   ├── openclaw-plugins/   # Plugin system
│   ├── openclaw-cli/       # CLI binary
│   └── openclaw-node/      # napi-rs bindings
└── docs/                   # Documentation
```

## Code Style

### Rust Guidelines

- **No `unsafe`**: All crates use `#![forbid(unsafe_code)]`
- **Error handling**: Use `Result<T, E>` with thiserror
- **Async**: Use tokio for async runtime
- **Documentation**: Add rustdoc comments to public APIs

### Lints

The workspace enforces these lints:

```toml
[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
```

### Formatting

```bash
# Format all code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check
```

## Testing

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p openclaw-core

# With output
cargo test --workspace -- --nocapture

# Doc tests
cargo test --workspace --doc
```

### Test Guidelines

- Place tests in `#[cfg(test)]` modules
- Use `tempfile` for filesystem tests
- Mock external services
- Test error conditions, not just happy paths

### Coverage

```bash
# Install coverage tool
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov --workspace --html
```

## Pull Request Workflow

### Before Submitting

1. **Build passes**: `cargo build --workspace`
2. **Tests pass**: `cargo test --workspace`
3. **Lints pass**: `cargo clippy --workspace -- -D warnings`
4. **Formatted**: `cargo fmt --all`
5. **Docs build**: `cargo doc --workspace --no-deps`

### PR Guidelines

- Keep PRs focused on a single change
- Include tests for new functionality
- Update documentation as needed
- Reference related issues

### Commit Messages

Use conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance

Examples:
```
feat(core): add session event filtering
fix(gateway): handle WebSocket disconnection
docs(readme): update architecture diagram
```

## Security

### Security Checklist

When contributing, ensure:

- [ ] No secrets in code or logs
- [ ] All external input validated
- [ ] New tools use sandboxing
- [ ] Error messages don't leak internal state
- [ ] Dependencies checked with `cargo audit`

### Reporting Vulnerabilities

Report security issues via GitHub Security Advisories, not public issues.

## Documentation

### Rustdoc

Add documentation comments to public items:

```rust
/// Validates message content for security constraints.
///
/// # Arguments
///
/// * `content` - The message content to validate
///
/// # Returns
///
/// `Ok(())` if valid, or `ValidationError` describing the issue.
///
/// # Examples
///
/// ```
/// use openclaw_core::validate_message_content;
///
/// assert!(validate_message_content("Hello").is_ok());
/// ```
pub fn validate_message_content(content: &str) -> Result<(), ValidationError> {
    // ...
}
```

### Building Docs

```bash
# Build and open documentation
cargo doc --workspace --open

# Include private items
cargo doc --workspace --document-private-items
```

## Priority Areas

We especially welcome contributions in these areas:

1. **Channel Adapters**: Telegram, Discord, Slack implementations
2. **Provider Clients**: Complete Anthropic, OpenAI API clients
3. **napi-rs Bindings**: Node.js integration
4. **Tests**: Increase coverage, add integration tests
5. **Documentation**: Examples, tutorials, API docs

## Getting Help

- Open an issue for questions
- Join the OpenClaw Discord community
- Check existing issues before creating new ones

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
