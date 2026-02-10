//! Input validation bindings.

use napi_derive::napi;

use openclaw_core::types::{AgentId, ChannelId, PeerId, PeerType, SessionKey};

/// Validate a message content string.
///
/// Performs:
/// - Length check (default max 100KB)
/// - Null byte removal
/// - Unicode normalization
///
/// Returns JSON: `{"valid": true, "sanitized": "..."}` or `{"valid": false, "error": "..."}`
#[napi]
#[must_use]
pub fn validate_message(content: String, max_length: Option<u32>) -> String {
    let max_len = max_length.unwrap_or(100_000) as usize;
    match openclaw_core::validation::validate_message_content(&content, max_len) {
        Ok(sanitized) => serde_json::json!({"valid": true, "sanitized": sanitized}).to_string(),
        Err(e) => serde_json::json!({"valid": false, "error": e.to_string()}).to_string(),
    }
}

/// Validate a file path for safety.
///
/// Checks for:
/// - Path traversal attempts
/// - Null bytes
/// - Absolute paths starting with /
///
/// Returns JSON: `{"valid": true}` or `{"valid": false, "error": "..."}`
#[napi]
#[must_use]
pub fn validate_path(path: String) -> String {
    match openclaw_core::validation::validate_path(&path) {
        Ok(()) => serde_json::json!({"valid": true}).to_string(),
        Err(e) => serde_json::json!({"valid": false, "error": e.to_string()}).to_string(),
    }
}

/// Build a session key from components.
///
/// # Arguments
///
/// * `agent_id` - The agent ID
/// * `channel` - Channel ID (e.g., "telegram", "discord")
/// * `account_id` - Account/bot ID on the channel
/// * `peer_type` - "dm", "group", "channel", or "thread"
/// * `peer_id` - The peer/user/group ID
#[napi]
#[must_use]
pub fn build_session_key(
    agent_id: String,
    channel: String,
    account_id: String,
    peer_type: String,
    peer_id: String,
) -> String {
    let pt = match peer_type.as_str() {
        "dm" => PeerType::Dm,
        "group" => PeerType::Group,
        "channel" => PeerType::Channel,
        "thread" => PeerType::Thread,
        _ => PeerType::Dm,
    };

    SessionKey::build(
        &AgentId::new(&agent_id),
        &ChannelId::new(&channel),
        &account_id,
        pt,
        &PeerId::new(&peer_id),
    )
    .as_ref()
    .to_string()
}
