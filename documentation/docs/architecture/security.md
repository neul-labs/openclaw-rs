# Security Model

!!! info "Community Implementation"
    A Rust implementation of [OpenClaw](https://github.com/openclaw/openclaw) by [Neul Labs](https://neullabs.com).

Security is a core consideration in openclaw-rs design.

---

## Threat Model

### Assets Protected

| Asset | Protection |
|-------|------------|
| API Keys | Encrypted at rest, memory protection |
| User Data | Access controls, sandboxing |
| System Access | Tool sandboxing, permission model |
| Network Traffic | TLS support, origin validation |

### Attack Vectors Considered

1. **Prompt Injection** - AI manipulated to bypass controls
2. **Credential Theft** - API keys exposed or stolen
3. **Tool Abuse** - Tools used for malicious purposes
4. **Privilege Escalation** - Breaking out of sandbox

---

## Credential Security

### Encrypted Storage

API keys are encrypted using AES-256-GCM:

```rust
pub struct CredentialStore {
    path: PathBuf,
    cipher: Aes256Gcm,
}

impl CredentialStore {
    pub fn store(&self, provider: &str, key: &str) -> Result<()> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, key.as_bytes())?;

        // Write encrypted data
        let entry = CredentialEntry {
            nonce: nonce.to_vec(),
            ciphertext,
            created_at: Utc::now(),
        };

        self.write_entry(provider, &entry)
    }
}
```

### Key Derivation

Master key derived from:

- Machine-specific identifier
- User-specific data
- Optional passphrase

### Memory Protection

Sensitive data in memory:

- Cleared after use (`zeroize` crate)
- Not written to logs
- Not included in error messages

---

## Tool Sandboxing

### Execution Environments

Tools execute in restricted environments:

=== "Linux"

    Uses `bubblewrap` for sandboxing:

    ```bash
    bwrap \
      --ro-bind /usr /usr \
      --ro-bind /lib /lib \
      --tmpfs /tmp \
      --unshare-net \
      --die-with-parent \
      command args
    ```

=== "macOS"

    Uses `sandbox-exec`:

    ```bash
    sandbox-exec -p '(version 1)
      (deny default)
      (allow file-read* (subpath "/usr"))
      (allow process-exec)
    ' command args
    ```

=== "Windows"

    Uses Windows Job Objects:

    - Process isolation
    - Resource limits
    - Handle inheritance blocked

### Permission Model

Tools declare required capabilities:

```rust
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub capabilities: Vec<Capability>,
}

pub enum Capability {
    FileRead { paths: Vec<PathPattern> },
    FileWrite { paths: Vec<PathPattern> },
    NetworkAccess { hosts: Vec<String> },
    ProcessExec { commands: Vec<String> },
}
```

### Path Restrictions

File access limited to:

- Workspace directory
- Explicitly allowed paths
- No access to home directory secrets

```json5
{
  "workspace": {
    "path": "~/.openclaw/workspace",
    "allowed_paths": ["~/projects"],
    "denied_paths": ["~/.ssh", "~/.aws"]
  }
}
```

---

## Network Security

### CORS Configuration

Control which origins can access the API:

```json5
{
  "gateway": {
    "cors_origins": [
      "http://localhost:3000",
      "https://app.example.com"
    ]
  }
}
```

### TLS Support

Enable HTTPS for production:

```json5
{
  "gateway": {
    "tls": {
      "cert": "/path/to/cert.pem",
      "key": "/path/to/key.pem"
    }
  }
}
```

### Rate Limiting

Prevent abuse with rate limits:

```json5
{
  "gateway": {
    "rate_limit": {
      "requests_per_minute": 60,
      "burst": 10
    }
  }
}
```

---

## Input Validation

### Request Validation

All inputs are validated:

```rust
impl Validator for MessageRequest {
    fn validate(&self) -> Result<(), ValidationError> {
        // Check message count
        if self.messages.is_empty() {
            return Err(ValidationError::EmptyMessages);
        }

        // Validate content
        for msg in &self.messages {
            msg.validate()?;
        }

        // Check token limits
        if self.max_tokens > MAX_ALLOWED_TOKENS {
            return Err(ValidationError::TokenLimit);
        }

        Ok(())
    }
}
```

### Tool Input Sanitization

Tool arguments are sanitized:

- Path traversal prevention
- Command injection blocking
- Size limits enforced

---

## Audit Logging

Security events are logged:

```rust
pub enum SecurityEvent {
    CredentialAccess { provider: String, success: bool },
    ToolExecution { tool: String, allowed: bool },
    AuthFailure { reason: String },
    RateLimitExceeded { client: String },
}

impl SecurityLogger {
    pub fn log(&self, event: SecurityEvent) {
        // Structured logging with timestamp
        tracing::info!(
            target: "security",
            event = ?event,
            timestamp = %Utc::now(),
        );
    }
}
```

---

## Security Checklist

### Deployment

- [ ] Use TLS in production
- [ ] Configure CORS appropriately
- [ ] Set rate limits
- [ ] Enable audit logging
- [ ] Use environment variables for secrets

### Configuration

- [ ] Review allowed paths
- [ ] Limit tool capabilities
- [ ] Set appropriate token limits
- [ ] Configure session timeouts

### Monitoring

- [ ] Monitor for unusual activity
- [ ] Review security logs
- [ ] Track rate limit hits
- [ ] Alert on auth failures

---

## Reporting Vulnerabilities

If you discover a security vulnerability:

1. **Do not** open a public issue
2. Email security concerns privately
3. Allow time for a fix before disclosure

---

## Next Steps

[:material-api: API Reference](../reference/core.md){ .md-button .md-button--primary }
[:material-swap-horizontal: OpenClaw Compatibility](../compatibility/index.md){ .md-button }
