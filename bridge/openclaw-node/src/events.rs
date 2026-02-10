//! Event store bindings for session event storage.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::path::PathBuf;
use std::sync::Arc;

use openclaw_core::events::{EventStore, SessionEvent, SessionEventKind};
use openclaw_core::types::{SessionKey, TokenUsage};

use crate::error::OpenClawError;

/// OpenClaw event store wrapper for Node.js.
///
/// Provides append-only event storage for session events with
/// CRDT projections for conflict-free state.
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
            .map_err(|e| OpenClawError::event_store_error(format!("EventStore open error: {e}")))?;

        Ok(Self {
            store: Arc::new(store),
        })
    }

    /// Append a session event and return the event ID.
    ///
    /// # Arguments
    ///
    /// * `session_key` - The session key
    /// * `agent_id` - The agent ID
    /// * `event_type` - Event type: "session_started", "message_received", "message_sent",
    ///                  "agent_response", "session_ended", "state_changed", "tool_called", "tool_result"
    /// * `data` - JSON data for the event
    #[napi]
    pub fn append_event(
        &self,
        session_key: String,
        agent_id: String,
        event_type: String,
        data: String,
    ) -> Result<String> {
        let data: serde_json::Value = serde_json::from_str(&data)
            .map_err(|e| OpenClawError::event_store_error(format!("Invalid JSON: {e}")))?;

        let kind = parse_event_kind(&event_type, &data)?;

        let event = SessionEvent::new(SessionKey::new(&session_key), agent_id, kind);

        let event_id = self
            .store
            .append(&event)
            .map_err(|e| OpenClawError::event_store_error(format!("Append error: {e}")))?;

        Ok(event_id.to_hex())
    }

    /// Get all events for a session as JSON.
    #[napi]
    pub fn get_events(&self, session_key: String) -> Result<String> {
        let events = self
            .store
            .get_events(&SessionKey::new(&session_key))
            .map_err(|e| OpenClawError::event_store_error(format!("Query error: {e}")))?;

        serde_json::to_string(&events)
            .map_err(|e| OpenClawError::event_store_error(format!("Serialization error: {e}")).into())
    }

    /// Get the session projection as JSON.
    ///
    /// The projection is a materialized view of the session state
    /// derived from the event stream.
    #[napi]
    pub fn get_projection(&self, session_key: String) -> Result<String> {
        let projection = self
            .store
            .get_projection(&SessionKey::new(&session_key))
            .map_err(|e| OpenClawError::event_store_error(format!("Projection error: {e}")))?;

        serde_json::to_string(&projection)
            .map_err(|e| OpenClawError::event_store_error(format!("Serialization error: {e}")).into())
    }

    /// List all session keys.
    #[napi]
    pub fn list_sessions(&self) -> Result<Vec<String>> {
        let sessions = self
            .store
            .list_sessions()
            .map_err(|e| OpenClawError::event_store_error(format!("List error: {e}")))?;

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
            .map_err(|e| OpenClawError::event_store_error(format!("Flush error: {e}")).into())
    }
}

/// Parse event type string into SessionEventKind.
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
            tokens: TokenUsage {
                input_tokens: data["tokens"]["input_tokens"].as_u64().unwrap_or(0),
                output_tokens: data["tokens"]["output_tokens"].as_u64().unwrap_or(0),
                cache_read_tokens: data["tokens"]["cache_read_tokens"].as_u64(),
                cache_write_tokens: data["tokens"]["cache_write_tokens"].as_u64(),
            },
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
        "tool_called" => Ok(SessionEventKind::ToolCalled {
            tool_name: data["tool_name"].as_str().unwrap_or("").to_string(),
            params: data.get("params").cloned().unwrap_or_default(),
        }),
        "tool_result" => Ok(SessionEventKind::ToolResult {
            tool_name: data["tool_name"].as_str().unwrap_or("").to_string(),
            result: data.get("result").cloned().unwrap_or_default(),
            success: data["success"].as_bool().unwrap_or(true),
        }),
        _ => Err(OpenClawError::event_store_error(format!(
            "Unknown event type: {event_type}"
        )).into()),
    }
}
