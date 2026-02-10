//! Configuration loading and validation bindings.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::path::PathBuf;

use openclaw_core::config::Config;

use crate::error::OpenClawError;

/// Load and parse an `OpenClaw` configuration file.
///
/// Returns the configuration as a JSON string.
#[napi]
pub fn load_config(path: String) -> Result<String> {
    let config = Config::load(&PathBuf::from(&path))
        .map_err(|e| OpenClawError::config_error(format!("Config load error: {e}")))?;

    serde_json::to_string_pretty(&config)
        .map_err(|e| OpenClawError::config_error(format!("Serialization error: {e}")).into())
}

/// Load the default configuration (~/.openclaw/openclaw.json).
///
/// Returns the configuration as a JSON string.
#[napi]
pub fn load_default_config() -> Result<String> {
    let config = Config::load_default()
        .map_err(|e| OpenClawError::config_error(format!("Config load error: {e}")))?;

    serde_json::to_string_pretty(&config)
        .map_err(|e| OpenClawError::config_error(format!("Serialization error: {e}")).into())
}

/// Validate configuration and return any errors.
///
/// Returns JSON: `{"valid": true, "errors": []}` or `{"valid": false, "errors": ["..."]}`
#[napi]
#[must_use]
pub fn validate_config(path: String) -> String {
    match Config::load(&PathBuf::from(&path)) {
        Ok(_) => serde_json::json!({"valid": true, "errors": []}).to_string(),
        Err(e) => serde_json::json!({"valid": false, "errors": [e.to_string()]}).to_string(),
    }
}
