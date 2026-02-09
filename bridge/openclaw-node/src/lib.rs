//! # OpenClaw Node.js Bridge
//!
//! napi-rs bindings to expose Rust core functionality to Node.js.
//!
//! Provides:
//! - Configuration loading and validation
//! - Event store operations (append, query, projection)
//! - Input validation
//! - Session key building

#![warn(missing_docs)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::path::PathBuf;
use std::sync::Arc;

use openclaw_core::config::Config;
use openclaw_core::events::{EventStore, SessionEvent, SessionEventKind};
use openclaw_core::types::{AgentId, ChannelId, PeerId, PeerType, SessionKey, TokenUsage};

// ---- Configuration ----

/// Load and parse an OpenClaw configuration file.
#[napi]
pub fn load_config(path: String) -> Result<String> {
    let config = Config::load(&PathBuf::from(&path))
        .map_err(|e| Error::from_reason(format!("Config load error: {e}")))?;

    serde_json::to_string_pretty(&config)
        .map_err(|e| Error::from_reason(format!("Serialization error: {e}")))
}

/// Load the default configuration (~/.openclaw/openclaw.json).
#[napi]
pub fn load_default_config() -> Result<String> {
    let config = Config::load_default()
        .map_err(|e| Error::from_reason(format!("Config load error: {e}")))?;

    serde_json::to_string_pretty(&config)
        .map_err(|e| Error::from_reason(format!("Serialization error: {e}")))
}

/// Validate configuration and return any errors.
#[napi]
pub fn validate_config(path: String) -> String {
    match Config::load(&PathBuf::from(&path)) {
        Ok(_) => serde_json::json!({"valid": true, "errors": []}).to_string(),
        Err(e) => serde_json::json!({"valid": false, "errors": [e.to_string()]}).to_string(),
    }
}

// ---- Event Store ----

/// OpenClaw event store wrapper for Node.js.
#[napi]
pub struct NodeEventStore {
    store: Arc<EventStore>,
}

#[napi]
impl NodeEventStore {
    /// Open or create an event store at the given path.
    #[napi(constructor)]
    pub fn new(path: String) -> Result<Self> {
        let store = EventStore::open(&PathBuf::from(&path))
            .map_err(|e| Error::from_reason(format!("EventStore open error: {e}")))?;

        Ok(Self {
            store: Arc::new(store),
        })
    }

    /// Append a session event and return the event ID.
    #[napi]
    pub fn append_event(
        &self,
        session_key: String,
        agent_id: String,
        event_type: String,
        data: String,
    ) -> Result<String> {
        let data: serde_json::Value = serde_json::from_str(&data)
            .map_err(|e| Error::from_reason(format!("Invalid JSON: {e}")))?;

        let kind = parse_event_kind(&event_type, &data)?;

        let event = SessionEvent::new(SessionKey::new(&session_key), agent_id, kind);

        let event_id = self
            .store
            .append(&event)
            .map_err(|e| Error::from_reason(format!("Append error: {e}")))?;

        Ok(event_id.to_hex())
    }

    /// Get all events for a session as JSON.
    #[napi]
    pub fn get_events(&self, session_key: String) -> Result<String> {
        let events = self
            .store
            .get_events(&SessionKey::new(&session_key))
            .map_err(|e| Error::from_reason(format!("Query error: {e}")))?;

        serde_json::to_string(&events)
            .map_err(|e| Error::from_reason(format!("Serialization error: {e}")))
    }

    /// Get the session projection as JSON.
    #[napi]
    pub fn get_projection(&self, session_key: String) -> Result<String> {
        let projection = self
            .store
            .get_projection(&SessionKey::new(&session_key))
            .map_err(|e| Error::from_reason(format!("Projection error: {e}")))?;

        serde_json::to_string(&projection)
            .map_err(|e| Error::from_reason(format!("Serialization error: {e}")))
    }

    /// List all session keys.
    #[napi]
    pub fn list_sessions(&self) -> Result<Vec<String>> {
        let sessions = self
            .store
            .list_sessions()
            .map_err(|e| Error::from_reason(format!("List error: {e}")))?;

        Ok(sessions
            .into_iter()
            .map(|s| s.as_ref().to_string())
            .collect())
    }

    /// Flush pending writes to disk.
    #[napi]
    pub fn flush(&self) -> Result<()> {
        self.store
            .flush()
            .map_err(|e| Error::from_reason(format!("Flush error: {e}")))
    }
}

// ---- Session Key Builder ----

/// Build a session key from components.
#[napi]
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

// ---- Validation ----

/// Validate a message content string (max 100KB by default).
#[napi]
pub fn validate_message(content: String, max_length: Option<u32>) -> String {
    let max_len = max_length.unwrap_or(100_000) as usize;
    match openclaw_core::validation::validate_message_content(&content, max_len) {
        Ok(sanitized) => {
            serde_json::json!({"valid": true, "sanitized": sanitized}).to_string()
        }
        Err(e) => {
            serde_json::json!({"valid": false, "error": e.to_string()}).to_string()
        }
    }
}

/// Validate a file path for safety (no traversal, etc.).
#[napi]
pub fn validate_path(path: String) -> String {
    match openclaw_core::validation::validate_path(&path) {
        Ok(()) => serde_json::json!({"valid": true}).to_string(),
        Err(e) => serde_json::json!({"valid": false, "error": e.to_string()}).to_string(),
    }
}

// ---- Internal helpers ----

fn parse_event_kind(
    event_type: &str,
    data: &serde_json::Value,
) -> Result<SessionEventKind> {
    match event_type {
        "session_started" => Ok(SessionEventKind::SessionStarted {
            channel: data["channel"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            peer_id: data["peer_id"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
        }),
        "message_received" => Ok(SessionEventKind::MessageReceived {
            content: data["content"].as_str().unwrap_or("").to_string(),
            attachments: vec![],
        }),
        "message_sent" => Ok(SessionEventKind::MessageSent {
            content: data["content"].as_str().unwrap_or("").to_string(),
            message_id: data["message_id"].as_str().unwrap_or("").to_string(),
        }),
        "agent_response" => Ok(SessionEventKind::AgentResponse {
            content: data["content"].as_str().unwrap_or("").to_string(),
            model: data["model"].as_str().unwrap_or("").to_string(),
            tokens: TokenUsage::default(),
        }),
        "session_ended" => Ok(SessionEventKind::SessionEnded {
            reason: data["reason"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
        }),
        "state_changed" => Ok(SessionEventKind::StateChanged {
            key: data["key"].as_str().unwrap_or("").to_string(),
            value: data.get("value").cloned().unwrap_or_default(),
        }),
        _ => Err(Error::from_reason(format!(
            "Unknown event type: {event_type}"
        ))),
    }
}
