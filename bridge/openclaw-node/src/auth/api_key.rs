//! Safe API key wrapper that prevents accidental logging.

use napi_derive::napi;

use openclaw_core::secrets::ApiKey;

/// Safe API key wrapper.
///
/// This wrapper prevents accidental logging or printing of API keys.
/// The key value is hidden from `toString()` and debug output.
///
/// ```javascript
/// const key = new NodeApiKey('sk-secret-key');
/// console.log(key.toString()); // "[REDACTED]"
/// console.log(key.exposeSecretForApiCall()); // "sk-secret-key"
/// ```
#[napi]
pub struct NodeApiKey {
    pub(crate) inner: ApiKey,
}

#[napi]
impl NodeApiKey {
    /// Create a new API key from a string.
    ///
    /// **Security Note**: Prefer loading keys from `CredentialStore`
    /// rather than hardcoding them.
    #[napi(constructor)]
    #[must_use]
    pub fn new(key: String) -> Self {
        Self {
            inner: ApiKey::new(key),
        }
    }

    /// Returns "[REDACTED]" - safe for logging.
    ///
    /// This method intentionally hides the key value to prevent
    /// accidental exposure in logs.
    #[napi]
    #[must_use]
    pub fn to_string(&self) -> String {
        "[REDACTED]".to_string()
    }

    /// Get the actual key value.
    ///
    /// **Warning**: Only use this when actually sending to an API.
    /// The verbose method name is intentional to discourage casual use.
    #[napi]
    #[must_use]
    pub fn expose_secret_for_api_call(&self) -> String {
        self.inner.expose().to_string()
    }

    /// Check if the key is empty.
    #[napi]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.expose().is_empty()
    }

    /// Get the length of the key.
    #[napi]
    #[must_use]
    pub fn length(&self) -> u32 {
        self.inner.expose().len() as u32
    }

    /// Check if the key starts with a prefix (e.g., "sk-ant-" for Anthropic).
    #[napi]
    #[must_use]
    pub fn starts_with(&self, prefix: String) -> bool {
        self.inner.expose().starts_with(&prefix)
    }
}

impl std::fmt::Debug for NodeApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeApiKey([REDACTED])")
    }
}
