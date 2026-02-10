# Contributing

!!! info "Community Implementation"
    A Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw) by [Neul Labs](https://neullabs.com).

Thank you for your interest in contributing to openclaw-rs!

---

## Ways to Contribute

### Code Contributions

- Bug fixes
- New features
- Performance improvements
- Documentation

### Non-Code Contributions

- Bug reports
- Feature requests
- Documentation improvements
- Testing and feedback

---

## Getting Started

### 1. Fork and Clone

```bash
# Fork on GitHub, then:
git clone https://github.com/YOUR_USERNAME/openclaw-rs
cd openclaw-rs
```

### 2. Set Up Development Environment

See [Development Setup](setup.md) for detailed instructions.

```bash
# Install dependencies
rustup update
cargo build

# Install UI dependencies
cd crates/openclaw-ui
npm install
```

### 3. Create a Branch

```bash
git checkout -b feature/my-feature
# or
git checkout -b fix/my-bugfix
```

---

## Development Workflow

### Running Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p openclaw-core

# With output
cargo test -- --nocapture
```

### Running Lints

```bash
# Clippy
cargo clippy --all-targets

# Format check
cargo fmt --check

# All checks
cargo clippy && cargo fmt --check && cargo test
```

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# With all features
cargo build --all-features
```

---

## Code Standards

### Rust Style

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting
- Address all `clippy` warnings
- Write documentation for public APIs

### Example

```rust
/// Creates a new message with the given content.
///
/// # Arguments
///
/// * `role` - The role of the message sender
/// * `content` - The message content
///
/// # Examples
///
/// ```
/// use openclaw_core::types::{Message, Role};
///
/// let msg = Message::new(Role::User, "Hello!");
/// assert_eq!(msg.content.as_text(), Some("Hello!"));
/// ```
pub fn new(role: Role, content: impl Into<Content>) -> Self {
    Self {
        role,
        content: content.into(),
        metadata: None,
    }
}
```

### Commit Messages

Use conventional commits:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `test`: Adding tests
- `chore`: Maintenance

Examples:
```
feat(providers): add streaming support for OpenAI

fix(agents): handle empty response from provider

docs(readme): update installation instructions
```

---

## Pull Request Process

### 1. Before Submitting

- [ ] Tests pass (`cargo test`)
- [ ] Lints pass (`cargo clippy`)
- [ ] Formatted (`cargo fmt`)
- [ ] Documentation updated
- [ ] Commit messages follow convention

### 2. PR Description

Include:
- What changed
- Why it changed
- How to test
- Breaking changes (if any)

### 3. Review Process

1. Automated checks run
2. Maintainer review
3. Address feedback
4. Merge when approved

---

## Issue Guidelines

### Bug Reports

Include:
- openclaw-rs version
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Error messages/logs

### Feature Requests

Include:
- Use case description
- Proposed solution
- Alternative solutions considered
- OpenClaw compatibility notes

---

## Code of Conduct

### Be Respectful

- Treat everyone with respect
- Be constructive in feedback
- Welcome newcomers

### Be Collaborative

- Help others learn
- Share knowledge
- Give credit

### Be Professional

- Focus on the work
- Accept constructive criticism
- Disagree respectfully

---

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

## Questions?

- Open a [GitHub Discussion](https://github.com/neul-labs/openclaw-rs/discussions)
- Check existing issues
- Read the documentation

---

## Next Steps

[:material-hammer-wrench: Development Setup](setup.md){ .md-button .md-button--primary }
[:material-map: Roadmap](roadmap.md){ .md-button }
