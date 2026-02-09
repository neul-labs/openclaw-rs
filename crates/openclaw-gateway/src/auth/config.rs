//! Authentication configuration.

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Default token expiry in hours.
const DEFAULT_TOKEN_EXPIRY_HOURS: u64 = 24;
/// Default refresh token expiry in days.
const DEFAULT_REFRESH_EXPIRY_DAYS: u64 = 7;

/// Authentication configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Whether authentication is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// JWT secret (hex-encoded). Auto-generated if not set.
    #[serde(default)]
    pub jwt_secret: Option<String>,

    /// Access token expiry in hours.
    #[serde(default = "default_token_expiry")]
    pub token_expiry_hours: u64,

    /// Refresh token expiry in days.
    #[serde(default = "default_refresh_expiry")]
    pub refresh_expiry_days: u64,

    /// Require auth for RPC calls.
    #[serde(default = "default_true")]
    pub require_auth_for_rpc: bool,

    /// Require auth for WebSocket connections.
    #[serde(default = "default_true")]
    pub require_auth_for_ws: bool,

    /// Methods that don't require authentication.
    #[serde(default = "default_public_methods")]
    pub public_methods: Vec<String>,
}

fn default_enabled() -> bool {
    true
}

fn default_true() -> bool {
    true
}

fn default_token_expiry() -> u64 {
    DEFAULT_TOKEN_EXPIRY_HOURS
}

fn default_refresh_expiry() -> u64 {
    DEFAULT_REFRESH_EXPIRY_DAYS
}

fn default_public_methods() -> Vec<String> {
    vec![
        "auth.login".to_string(),
        "setup.status".to_string(),
        "setup.init".to_string(),
        "system.health".to_string(),
        "system.version".to_string(),
    ]
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            jwt_secret: None,
            token_expiry_hours: default_token_expiry(),
            refresh_expiry_days: default_refresh_expiry(),
            require_auth_for_rpc: default_true(),
            require_auth_for_ws: default_true(),
            public_methods: default_public_methods(),
        }
    }
}

impl AuthConfig {
    /// Create a new auth config builder.
    #[must_use]
    pub fn builder() -> AuthConfigBuilder {
        AuthConfigBuilder::default()
    }

    /// Get token expiry as Duration.
    #[must_use]
    pub fn token_expiry(&self) -> Duration {
        Duration::from_secs(self.token_expiry_hours * 3600)
    }

    /// Get refresh token expiry as Duration.
    #[must_use]
    pub fn refresh_expiry(&self) -> Duration {
        Duration::from_secs(self.refresh_expiry_days * 24 * 3600)
    }

    /// Check if a method is public (doesn't require auth).
    #[must_use]
    pub fn is_public_method(&self, method: &str) -> bool {
        self.public_methods.iter().any(|m| m == method)
    }

    /// Load config from environment variables (overrides).
    #[must_use]
    pub fn with_env_overrides(mut self) -> Self {
        if let Ok(secret) = std::env::var("OPENCLAW_JWT_SECRET") {
            self.jwt_secret = Some(secret);
        }

        if std::env::var("OPENCLAW_AUTH_DISABLED")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
        {
            self.enabled = false;
        }

        self
    }
}

/// Builder for `AuthConfig`.
#[derive(Debug, Default)]
pub struct AuthConfigBuilder {
    config: AuthConfig,
}

impl AuthConfigBuilder {
    /// Set whether auth is enabled.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Set the JWT secret.
    #[must_use]
    pub fn jwt_secret(mut self, secret: impl Into<String>) -> Self {
        self.config.jwt_secret = Some(secret.into());
        self
    }

    /// Set token expiry in hours.
    #[must_use]
    pub fn token_expiry_hours(mut self, hours: u64) -> Self {
        self.config.token_expiry_hours = hours;
        self
    }

    /// Set refresh token expiry in days.
    #[must_use]
    pub fn refresh_expiry_days(mut self, days: u64) -> Self {
        self.config.refresh_expiry_days = days;
        self
    }

    /// Set whether RPC requires auth.
    #[must_use]
    pub fn require_auth_for_rpc(mut self, required: bool) -> Self {
        self.config.require_auth_for_rpc = required;
        self
    }

    /// Set whether WebSocket requires auth.
    #[must_use]
    pub fn require_auth_for_ws(mut self, required: bool) -> Self {
        self.config.require_auth_for_ws = required;
        self
    }

    /// Add a public method (doesn't require auth).
    #[must_use]
    pub fn public_method(mut self, method: impl Into<String>) -> Self {
        self.config.public_methods.push(method.into());
        self
    }

    /// Build the config.
    #[must_use]
    pub fn build(self) -> AuthConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AuthConfig::default();
        assert!(config.enabled);
        assert!(config.jwt_secret.is_none());
        assert_eq!(config.token_expiry_hours, 24);
        assert_eq!(config.refresh_expiry_days, 7);
    }

    #[test]
    fn test_public_methods() {
        let config = AuthConfig::default();
        assert!(config.is_public_method("auth.login"));
        assert!(config.is_public_method("system.health"));
        assert!(!config.is_public_method("session.create"));
    }

    #[test]
    fn test_builder() {
        let config = AuthConfig::builder()
            .enabled(false)
            .token_expiry_hours(12)
            .build();

        assert!(!config.enabled);
        assert_eq!(config.token_expiry_hours, 12);
    }

    #[test]
    fn test_durations() {
        let config = AuthConfig::default();
        assert_eq!(config.token_expiry(), Duration::from_secs(24 * 3600));
        assert_eq!(config.refresh_expiry(), Duration::from_secs(7 * 24 * 3600));
    }
}
