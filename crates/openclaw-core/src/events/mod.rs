//! Event-sourced session storage (grite pattern).
//!
//! Sessions are stored as append-only event logs with CRDT projections
//! for materialized views. Uses sled for fast local storage.

use blake2::{Blake2b, Digest, digest::consts::U32};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

use crate::types::{ChannelId, SessionKey, TokenUsage};

/// Event store errors.
#[derive(Error, Debug)]
pub enum EventStoreError {
    /// Storage error.
    #[error("Storage error: {0}")]
    Storage(#[from] sled::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Event not found.
    #[error("Event not found: {0}")]
    NotFound(String),
}

/// Unique event identifier (`BLAKE2b` hash).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub [u8; 32]);

impl EventId {
    /// Generate event ID from content.
    #[must_use]
    pub fn from_content(content: &[u8]) -> Self {
        let mut hasher = Blake2b::<U32>::new();
        hasher.update(content);
        let result = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(&result);
        Self(id)
    }

    /// Convert to hex string.
    #[must_use]
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.to_hex()[..12])
    }
}

/// A session event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    /// Unique event identifier.
    pub id: EventId,
    /// Session this event belongs to.
    pub session_key: SessionKey,
    /// Agent that processed this event.
    pub agent_id: String,
    /// When the event occurred.
    pub timestamp: DateTime<Utc>,
    /// Event payload.
    pub kind: SessionEventKind,
}

impl SessionEvent {
    /// Create a new session event.
    #[must_use]
    pub fn new(session_key: SessionKey, agent_id: String, kind: SessionEventKind) -> Self {
        let timestamp = Utc::now();
        let content = format!("{session_key}:{agent_id}:{timestamp}:{kind:?}");
        let id = EventId::from_content(content.as_bytes());

        Self {
            id,
            session_key,
            agent_id,
            timestamp,
            kind,
        }
    }
}

/// Types of session events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionEventKind {
    /// Session started.
    SessionStarted {
        /// Channel the session is on.
        channel: String,
        /// Peer ID.
        peer_id: String,
    },

    /// Message received from peer.
    MessageReceived {
        /// Message content.
        content: String,
        /// Attachment metadata.
        attachments: Vec<AttachmentMeta>,
    },

    /// Message sent to peer.
    MessageSent {
        /// Message content.
        content: String,
        /// Platform message ID.
        message_id: String,
    },

    /// Tool was called.
    ToolCalled {
        /// Tool name.
        tool_name: String,
        /// Tool parameters.
        params: serde_json::Value,
    },

    /// Tool returned result.
    ToolResult {
        /// Tool name.
        tool_name: String,
        /// Tool result.
        result: serde_json::Value,
        /// Whether tool succeeded.
        success: bool,
    },

    /// Agent produced a response.
    AgentResponse {
        /// Response content.
        content: String,
        /// Model used.
        model: String,
        /// Token usage.
        tokens: TokenUsage,
    },

    /// Session ended.
    SessionEnded {
        /// Reason for ending.
        reason: String,
    },

    /// Session state changed.
    StateChanged {
        /// State key.
        key: String,
        /// New value.
        value: serde_json::Value,
    },
}

/// Attachment metadata for events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentMeta {
    /// Attachment type.
    pub kind: String,
    /// MIME type.
    pub mime_type: Option<String>,
    /// File size.
    pub size: Option<u64>,
}

/// Session state for projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionState {
    /// Session is active.
    Active,
    /// Session is paused.
    Paused,
    /// Session has ended.
    Ended,
}

/// A message in session history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionMessage {
    /// Message from the peer.
    Inbound(String),
    /// Message to the peer.
    Outbound(String),
    /// Tool call.
    Tool { name: String, result: String },
}

/// CRDT projection for session state.
///
/// This is a materialized view derived from applying events.
/// Supports CRDT merge for conflict resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionProjection {
    /// Session identifier.
    pub session_key: SessionKey,
    /// Agent ID.
    pub agent_id: String,
    /// Channel.
    pub channel: ChannelId,
    /// Peer ID.
    pub peer_id: String,
    /// Current state.
    pub state: SessionState,
    /// Total message count.
    pub message_count: u64,
    /// Last activity timestamp.
    pub last_activity: DateTime<Utc>,
    /// Message history.
    pub messages: Vec<SessionMessage>,
    /// Custom state key-value pairs.
    #[serde(default)]
    pub custom_state: std::collections::HashMap<String, serde_json::Value>,
    /// Last event ID applied.
    pub last_event_id: Option<EventId>,
}

impl SessionProjection {
    /// Create a new empty projection.
    #[must_use]
    pub fn new(
        session_key: SessionKey,
        agent_id: String,
        channel: ChannelId,
        peer_id: String,
    ) -> Self {
        Self {
            session_key,
            agent_id,
            channel,
            peer_id,
            state: SessionState::Active,
            message_count: 0,
            last_activity: Utc::now(),
            messages: Vec::new(),
            custom_state: std::collections::HashMap::new(),
            last_event_id: None,
        }
    }

    /// Apply an event to update the projection.
    pub fn apply(&mut self, event: &SessionEvent) {
        match &event.kind {
            SessionEventKind::SessionStarted { .. } => {
                self.state = SessionState::Active;
            }
            SessionEventKind::MessageReceived { content, .. } => {
                self.messages.push(SessionMessage::Inbound(content.clone()));
                self.message_count += 1;
            }
            SessionEventKind::MessageSent { content, .. } => {
                self.messages
                    .push(SessionMessage::Outbound(content.clone()));
            }
            SessionEventKind::ToolCalled { tool_name, .. } => {
                // Tool calls are recorded but don't add to message history yet
                tracing::debug!(tool = %tool_name, "Tool called");
            }
            SessionEventKind::ToolResult {
                tool_name, result, ..
            } => {
                let result_str = serde_json::to_string(result).unwrap_or_default();
                self.messages.push(SessionMessage::Tool {
                    name: tool_name.clone(),
                    result: result_str,
                });
            }
            SessionEventKind::AgentResponse { content, .. } => {
                self.messages
                    .push(SessionMessage::Outbound(content.clone()));
            }
            SessionEventKind::SessionEnded { .. } => {
                self.state = SessionState::Ended;
            }
            SessionEventKind::StateChanged { key, value } => {
                self.custom_state.insert(key.clone(), value.clone());
            }
        }

        self.last_activity = event.timestamp;
        self.last_event_id = Some(event.id.clone());
    }

    /// CRDT merge with another projection.
    ///
    /// Uses last-write-wins for scalar fields, union for messages.
    pub fn merge(&mut self, other: &Self) {
        // Last-write-wins for activity timestamp
        if other.last_activity > self.last_activity {
            self.state = other.state;
            self.last_activity = other.last_activity;
            self.last_event_id = other.last_event_id.clone();
        }

        // Union of messages (deduplicate by content hash would be ideal)
        // For now, take the longer history
        if other.messages.len() > self.messages.len() {
            self.messages = other.messages.clone();
            self.message_count = other.message_count;
        }

        // Merge custom state (last-write-wins per key)
        for (key, value) in &other.custom_state {
            self.custom_state.insert(key.clone(), value.clone());
        }
    }
}

/// Event store backed by sled.
pub struct EventStore {
    db: sled::Db,
    events_tree: sled::Tree,
    sessions_tree: sled::Tree,
}

impl EventStore {
    /// Open or create an event store.
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be opened.
    pub fn open(path: &Path) -> Result<Self, EventStoreError> {
        let db = sled::open(path)?;
        let events_tree = db.open_tree("events")?;
        let sessions_tree = db.open_tree("sessions")?;

        Ok(Self {
            db,
            events_tree,
            sessions_tree,
        })
    }

    /// Append an event to a session's event log.
    ///
    /// # Errors
    ///
    /// Returns error if storage fails.
    pub fn append(&self, event: &SessionEvent) -> Result<EventId, EventStoreError> {
        let event_key = format!("{}:{}", event.session_key, event.id.to_hex());
        let event_data = serde_json::to_vec(event)?;

        self.events_tree.insert(event_key.as_bytes(), event_data)?;

        // Update session projection
        self.update_projection(event)?;

        Ok(event.id.clone())
    }

    /// Get all events for a session.
    ///
    /// # Errors
    ///
    /// Returns error if storage read fails.
    pub fn get_events(
        &self,
        session_key: &SessionKey,
    ) -> Result<Vec<SessionEvent>, EventStoreError> {
        let prefix = format!("{session_key}:");
        let mut events = Vec::new();

        for result in self.events_tree.scan_prefix(prefix.as_bytes()) {
            let (_, value) = result?;
            let event: SessionEvent = serde_json::from_slice(&value)?;
            events.push(event);
        }

        // Sort by timestamp
        events.sort_by_key(|e| e.timestamp);
        Ok(events)
    }

    /// Get events since a specific timestamp.
    ///
    /// # Errors
    ///
    /// Returns error if storage read fails.
    pub fn get_events_since(
        &self,
        session_key: &SessionKey,
        since: DateTime<Utc>,
    ) -> Result<Vec<SessionEvent>, EventStoreError> {
        let events = self.get_events(session_key)?;
        Ok(events.into_iter().filter(|e| e.timestamp > since).collect())
    }

    /// Get the session projection.
    ///
    /// # Errors
    ///
    /// Returns error if storage read fails or projection not found.
    pub fn get_projection(
        &self,
        session_key: &SessionKey,
    ) -> Result<SessionProjection, EventStoreError> {
        let key = session_key.as_ref().as_bytes();

        match self.sessions_tree.get(key)? {
            Some(data) => Ok(serde_json::from_slice(&data)?),
            None => Err(EventStoreError::NotFound(session_key.to_string())),
        }
    }

    /// List all session keys.
    ///
    /// # Errors
    ///
    /// Returns error if storage read fails.
    pub fn list_sessions(&self) -> Result<Vec<SessionKey>, EventStoreError> {
        let mut sessions = Vec::new();

        for result in &self.sessions_tree {
            let (key, _) = result?;
            if let Ok(key_str) = std::str::from_utf8(&key) {
                sessions.push(SessionKey::new(key_str));
            }
        }

        Ok(sessions)
    }

    /// Update the session projection after appending an event.
    fn update_projection(&self, event: &SessionEvent) -> Result<(), EventStoreError> {
        let key = event.session_key.as_ref().as_bytes();

        let mut projection = match self.sessions_tree.get(key)? {
            Some(data) => serde_json::from_slice(&data)?,
            None => {
                // Create new projection from SessionStarted event
                if let SessionEventKind::SessionStarted { channel, peer_id } = &event.kind {
                    SessionProjection::new(
                        event.session_key.clone(),
                        event.agent_id.clone(),
                        ChannelId::new(channel),
                        peer_id.clone(),
                    )
                } else {
                    // Create with defaults if no SessionStarted
                    SessionProjection::new(
                        event.session_key.clone(),
                        event.agent_id.clone(),
                        ChannelId::new("unknown"),
                        "unknown".to_string(),
                    )
                }
            }
        };

        projection.apply(event);

        let projection_data = serde_json::to_vec(&projection)?;
        self.sessions_tree.insert(key, projection_data)?;

        Ok(())
    }

    /// Flush all pending writes to disk.
    ///
    /// # Errors
    ///
    /// Returns error if flush fails.
    pub fn flush(&self) -> Result<(), EventStoreError> {
        self.db.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AgentId;
    use tempfile::tempdir;

    #[test]
    fn test_event_id_generation() {
        let id1 = EventId::from_content(b"test content");
        let id2 = EventId::from_content(b"test content");
        let id3 = EventId::from_content(b"different content");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_session_event_creation() {
        let session_key = SessionKey::new("test-session");
        let event = SessionEvent::new(
            session_key.clone(),
            "agent1".to_string(),
            SessionEventKind::MessageReceived {
                content: "Hello".to_string(),
                attachments: vec![],
            },
        );

        assert_eq!(event.session_key, session_key);
        assert_eq!(event.agent_id, "agent1");
    }

    #[test]
    fn test_projection_apply() {
        let mut projection = SessionProjection::new(
            SessionKey::new("test"),
            "agent".to_string(),
            ChannelId::telegram(),
            "user123".to_string(),
        );

        let event = SessionEvent::new(
            SessionKey::new("test"),
            "agent".to_string(),
            SessionEventKind::MessageReceived {
                content: "Hello".to_string(),
                attachments: vec![],
            },
        );

        projection.apply(&event);

        assert_eq!(projection.message_count, 1);
        assert_eq!(projection.messages.len(), 1);
    }

    #[test]
    fn test_event_store_roundtrip() {
        let temp = tempdir().unwrap();
        let store = EventStore::open(temp.path()).unwrap();

        let session_key = SessionKey::build(
            &AgentId::default_agent(),
            &ChannelId::telegram(),
            "bot123",
            crate::types::PeerType::Dm,
            &crate::types::PeerId::new("user456"),
        );

        // Start session
        let start_event = SessionEvent::new(
            session_key.clone(),
            "default".to_string(),
            SessionEventKind::SessionStarted {
                channel: "telegram".to_string(),
                peer_id: "user456".to_string(),
            },
        );
        store.append(&start_event).unwrap();

        // Add message
        let msg_event = SessionEvent::new(
            session_key.clone(),
            "default".to_string(),
            SessionEventKind::MessageReceived {
                content: "Hello, agent!".to_string(),
                attachments: vec![],
            },
        );
        store.append(&msg_event).unwrap();

        // Verify events
        let events = store.get_events(&session_key).unwrap();
        assert_eq!(events.len(), 2);

        // Verify projection
        let projection = store.get_projection(&session_key).unwrap();
        assert_eq!(projection.message_count, 1);
        assert_eq!(projection.state, SessionState::Active);
    }
}
