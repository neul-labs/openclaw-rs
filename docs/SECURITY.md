# Security

OpenClaw Rust Core is designed with security as a foundational requirement, not an afterthought.

## Security Principles

| Principle | Description |
|-----------|-------------|
| **Defense in Depth** | Multiple layers of security controls |
| **Least Privilege** | Components have minimal required permissions |
| **Fail Secure** | Errors default to denial, not exposure |
| **Zero Trust** | All external input is validated |
| **Secure by Default** | Safe configuration out of the box |

## Threat Model

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              THREAT SOURCES                                      │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐│
│  │  Malicious   │  │  Compromised │  │   Network    │  │  Malicious Plugin/   ││
│  │   Users      │  │   Channels   │  │   Attackers  │  │  Tool Code           ││
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └──────────┬───────────┘│
│         │                 │                 │                     │            │
│         ▼                 ▼                 ▼                     ▼            │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │                        ATTACK VECTORS                                    │   │
│  │  • Prompt injection via messages                                         │   │
│  │  • Oversized payloads (DoS)                                              │   │
│  │  • Malformed JSON/content                                                │   │
│  │  • Command injection via tools                                           │   │
│  │  • Path traversal in file operations                                     │   │
│  │  • Secret exfiltration                                                   │   │
│  │  • Sandbox escape                                                        │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│                                     │                                           │
│                                     ▼                                           │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │                        DEFENSES                                          │   │
│  │  ✓ Input validation at boundaries                                        │   │
│  │  ✓ Size limits on all inputs                                             │   │
│  │  ✓ Sandboxed tool execution                                              │   │
│  │  ✓ Secret redaction in logs                                              │   │
│  │  ✓ Encrypted credential storage                                          │   │
│  │  ✓ Rate limiting                                                         │   │
│  │  ✓ Allowlist-based access control                                        │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## Input Validation

All external input passes through validation in `openclaw-core`:

```rust
// Size limits
pub const MAX_MESSAGE_SIZE: usize = 100_000;      // 100KB
pub const MAX_JSON_DEPTH: usize = 32;
pub const MAX_ATTACHMENT_SIZE: u64 = 50_000_000;  // 50MB
pub const MAX_ATTACHMENTS: usize = 10;

// Validation function
pub fn validate_message_content(content: &str) -> Result<(), ValidationError> {
    // Check size
    if content.len() > MAX_MESSAGE_SIZE {
        return Err(ValidationError::ContentTooLarge { ... });
    }

    // Check for null bytes
    if content.contains('\0') {
        return Err(ValidationError::InvalidContent { ... });
    }

    // Sanitize control characters
    // ...
}
```

### Validated Properties

| Property | Check | Limit |
|----------|-------|-------|
| Message size | Length check | 100KB |
| JSON depth | Recursive depth | 32 levels |
| Attachment size | File size | 50MB |
| Attachment count | Array length | 10 |
| Control characters | Sanitization | Stripped |
| Null bytes | Rejection | Not allowed |

## Secrets Management

### ApiKey Wrapper

The `ApiKey` type prevents accidental secret exposure:

```rust
pub struct ApiKey(SecretBox<str>);

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApiKey([REDACTED])")  // Never prints the actual key
    }
}

impl std::fmt::Display for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}
```

### Credential Store

Credentials are encrypted at rest using AES-256-GCM:

```
┌─────────────────────────────────────────────────────────────────┐
│                    CredentialStore                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Master Password ──▶ Argon2id ──▶ 256-bit Key                   │
│                                        │                         │
│                                        ▼                         │
│  Plaintext ──▶ AES-256-GCM ──▶ [Nonce || Ciphertext || Tag]     │
│                                        │                         │
│                                        ▼                         │
│                              ~/.openclaw/credentials/            │
│                                   ├── anthropic.enc              │
│                                   ├── openai.enc                 │
│                                   └── ...                        │
│                                                                  │
│  File permissions: 0600 (owner read/write only)                 │
└─────────────────────────────────────────────────────────────────┘
```

### Secret Scrubbing

Logs and error messages are scrubbed for secrets:

```rust
pub fn scrub_secrets(text: &str, patterns: &[&str]) -> String;

pub const COMMON_SECRET_PATTERNS: &[&str] = &[
    "api_key=",
    "token=",
    "secret=",
    "password=",
    "Authorization: Bearer ",
    // ...
];
```

## Sandboxing

Tool execution is isolated using platform-specific sandboxing:

### Linux (bubblewrap)

```rust
Command::new("bwrap")
    .arg("--ro-bind").arg("/usr").arg("/usr")
    .arg("--ro-bind").arg("/lib").arg("/lib")
    .arg("--ro-bind").arg("/lib64").arg("/lib64")
    .arg("--bind").arg(&workspace).arg(&workspace)
    .arg("--proc").arg("/proc")
    .arg("--dev").arg("/dev")
    .arg("--unshare-net")   // Network isolation
    .arg("--unshare-pid")   // PID namespace
    .arg("--die-with-parent")
    .arg("--")
    .arg(&command)
```

### macOS (sandbox-exec)

```rust
const SANDBOX_PROFILE: &str = r#"
(version 1)
(deny default)
(allow file-read* (subpath "/usr"))
(allow file-read* file-write* (subpath "{workspace}"))
(allow process-exec)
(allow process-fork)
(deny network*)
"#;

Command::new("sandbox-exec")
    .arg("-p").arg(&profile)
    .arg(&command)
```

### Windows (Job Objects)

```rust
// Job object with restrictions:
// - No child process creation outside job
// - CPU time limits
// - Memory limits
// - No UI access
```

### Sandbox Levels

| Level | Network | Filesystem | Processes |
|-------|---------|------------|-----------|
| `None` | Allowed | Full access | Unrestricted |
| `Relaxed` | Allowed | Workspace only | Restricted |
| `Strict` | Blocked | Workspace only | Isolated namespace |

## Authentication

### OAuth Flow

```
┌────────┐     ┌─────────┐     ┌──────────────┐     ┌──────────┐
│ Client │────▶│ Gateway │────▶│ OAuth Server │────▶│ Provider │
└────────┘     └─────────┘     └──────────────┘     └──────────┘
                   │                   │
                   │ ◀─────────────────┘
                   │   (access_token, refresh_token)
                   │
                   ▼
            ┌─────────────┐
            │ Credential  │
            │   Store     │  (encrypted at rest)
            └─────────────┘
```

### Token Security

- Access tokens: Short-lived (1 hour default)
- Refresh tokens: Encrypted storage, rotation on use
- Scopes: Minimal required permissions

## Rate Limiting

```rust
pub struct GatewayRateLimiter {
    limits: HashMap<String, RateLimit>,
    window_seconds: u64,
}

// Default limits
const DEFAULT_REQUESTS_PER_MINUTE: u32 = 60;
const DEFAULT_MESSAGES_PER_MINUTE: u32 = 30;
```

## Audit Logging

Security-relevant events are logged:

```rust
// Logged events:
// - Authentication attempts (success/failure)
// - Authorization decisions
// - Tool executions
// - Configuration changes
// - Rate limit triggers

tracing::info!(
    event = "auth_attempt",
    user = %peer_id,
    channel = %channel,
    success = result.is_ok(),
);
```

## Dependency Security

### cargo-deny Configuration

```toml
# deny.toml
[advisories]
db-path = "~/.cargo/advisory-db"
vulnerability = "deny"
unmaintained = "warn"

[licenses]
allow = ["MIT", "Apache-2.0", "BSD-3-Clause"]
copyleft = "deny"

[bans]
multiple-versions = "warn"
deny = [
    { name = "openssl" },  # Prefer rustls
]
```

### Safe Dependencies

| Category | Crate | Reason |
|----------|-------|--------|
| Crypto | `aes-gcm` | Pure Rust, audited |
| Crypto | `blake2` | Pure Rust, well-tested |
| Secrets | `secrecy` | Zeroize on drop |
| TLS | `rustls` | Memory-safe TLS |
| HTTP | `axum` | Tokio ecosystem, well-maintained |
| Storage | `sled` | Embedded, ACID guarantees |

## Security Checklist

When contributing, verify:

- [ ] All external input is validated
- [ ] No secrets in logs or error messages
- [ ] New tools use sandboxing
- [ ] No `unsafe` code (or justified exception)
- [ ] Dependencies checked against advisories
- [ ] Error messages don't leak internal state
- [ ] Rate limits applied to new endpoints
- [ ] File operations use safe path handling

## Reporting Vulnerabilities

If you discover a security vulnerability, please report it via:

1. **GitHub Security Advisories** (preferred)
2. Email to security@openclaw.ai

Please do not open public issues for security vulnerabilities.
