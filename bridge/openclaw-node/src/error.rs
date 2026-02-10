//! Structured error types for JavaScript consumption.

use serde::{Deserialize, Serialize};

/// Structured error for JavaScript consumption.
///
/// This provides rich error information that can be easily handled
/// in JavaScript code with error codes, status, and retry hints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClawError {
    /// Error code for programmatic handling (e.g., "PROVIDER_ERROR", "AUTH_ERROR")
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional context as JSON (optional)
    pub details: Option<serde_json::Value>,
    /// HTTP status code if applicable (for API errors)
    pub status: Option<u16>,
    /// Retry after seconds (for rate limits)
    pub retry_after: Option<u32>,
}

impl OpenClawError {
    /// Create a new error with just code and message.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
            status: None,
            retry_after: None,
        }
    }

    /// Create an error from a provider error.
    pub fn from_provider_error(e: openclaw_providers::ProviderError) -> Self {
        use openclaw_providers::ProviderError;

        match &e {
            ProviderError::Api { status, message } => Self {
                code: "PROVIDER_API_ERROR".to_string(),
                message: message.clone(),
                details: None,
                status: Some(*status),
                retry_after: None,
            },
            ProviderError::RateLimited { retry_after_secs } => Self {
                code: "RATE_LIMITED".to_string(),
                message: "Rate limited by provider".to_string(),
                details: None,
                status: Some(429),
                retry_after: Some(*retry_after_secs as u32),
            },
            ProviderError::Network(err) => Self {
                code: "NETWORK_ERROR".to_string(),
                message: err.to_string(),
                details: None,
                status: None,
                retry_after: None,
            },
            ProviderError::Config(msg) => Self {
                code: "CONFIG_ERROR".to_string(),
                message: msg.clone(),
                details: None,
                status: Some(400),
                retry_after: None,
            },
            ProviderError::Serialization(err) => Self {
                code: "SERIALIZATION_ERROR".to_string(),
                message: err.to_string(),
                details: None,
                status: None,
                retry_after: None,
            },
        }
    }

    /// Create an error from a credential error.
    pub fn from_credential_error(e: openclaw_core::secrets::CredentialError) -> Self {
        use openclaw_core::secrets::CredentialError;

        match &e {
            CredentialError::NotFound(name) => Self {
                code: "CREDENTIAL_NOT_FOUND".to_string(),
                message: format!("Credential not found: {name}"),
                details: None,
                status: Some(404),
                retry_after: None,
            },
            CredentialError::Crypto(msg) => Self {
                code: "CRYPTO_ERROR".to_string(),
                message: msg.clone(),
                details: None,
                status: None,
                retry_after: None,
            },
            _ => Self {
                code: "CREDENTIAL_ERROR".to_string(),
                message: e.to_string(),
                details: None,
                status: None,
                retry_after: None,
            },
        }
    }

    /// Create a config error.
    pub fn config_error(message: impl Into<String>) -> Self {
        Self::new("CONFIG_ERROR", message)
    }

    /// Create an event store error.
    pub fn event_store_error(message: impl Into<String>) -> Self {
        Self::new("EVENT_STORE_ERROR", message)
    }

    /// Create a validation error.
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new("VALIDATION_ERROR", message)
    }

    /// Create a tool error.
    pub fn tool_error(message: impl Into<String>) -> Self {
        Self::new("TOOL_ERROR", message)
    }

    /// Create an agent error.
    pub fn agent_error(message: impl Into<String>) -> Self {
        Self::new("AGENT_ERROR", message)
    }
}

impl From<OpenClawError> for napi::Error {
    fn from(e: OpenClawError) -> Self {
        // Serialize the full error as JSON for structured handling in JS
        let json = serde_json::to_string(&e).unwrap_or_else(|_| e.message.clone());
        napi::Error::from_reason(json)
    }
}

/// Helper to convert any error to a napi::Error with an OpenClawError.
pub fn to_napi_error(code: &str, e: impl std::fmt::Display) -> napi::Error {
    OpenClawError::new(code, e.to_string()).into()
}
