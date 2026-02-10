//! WebSocket UI events for real-time updates.
//!
//! This module provides a broadcast system for pushing events to connected
//! WebSocket clients.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

/// UI event types that can be broadcast to connected clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UiEvent {
    /// A new session was created.
    SessionCreated {
        /// Session key.
        session_key: String,
        /// Agent ID.
        agent_id: String,
        /// Channel.
        channel: String,
        /// Peer ID.
        peer_id: String,
    },

    /// A session was updated.
    SessionUpdated {
        /// Session key.
        session_key: String,
        /// Update type.
        update: SessionUpdate,
    },

    /// A message was received from a peer.
    MessageReceived {
        /// Session key.
        session_key: String,
        /// Message content.
        content: String,
        /// Peer ID.
        peer_id: String,
    },

    /// A message was sent to a peer.
    MessageSent {
        /// Session key.
        session_key: String,
        /// Message content.
        content: String,
    },

    /// A tool was executed.
    ToolExecuted {
        /// Session key.
        session_key: String,
        /// Tool name.
        tool: String,
        /// Tool result.
        result: serde_json::Value,
        /// Whether the tool succeeded.
        success: bool,
    },

    /// Channel status changed.
    ChannelStatusChanged {
        /// Channel ID.
        channel_id: String,
        /// Whether connected.
        connected: bool,
        /// Error message if any.
        error: Option<String>,
    },

    /// Heartbeat event.
    Heartbeat {
        /// Timestamp.
        timestamp: DateTime<Utc>,
    },
}

/// Session update types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionUpdate {
    /// Session state changed.
    StateChanged {
        /// The new state name.
        new_state: String,
    },
    /// Message count updated.
    MessageCount {
        /// The current message count.
        count: u64,
    },
    /// Session ended.
    Ended {
        /// The reason for ending.
        reason: String,
    },
}

/// A wrapper for UI events with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiEventEnvelope {
    /// Event ID.
    pub id: String,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
    /// The event payload.
    pub event: UiEvent,
}

impl UiEventEnvelope {
    /// Create a new event envelope.
    #[must_use]
    pub fn new(event: UiEvent) -> Self {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 8];
        rng.fill_bytes(&mut bytes);

        Self {
            id: hex::encode(bytes),
            timestamp: Utc::now(),
            event,
        }
    }
}

/// Default channel capacity for event broadcasts.
const DEFAULT_CHANNEL_CAPACITY: usize = 256;

/// Event broadcaster for distributing UI events to subscribers.
pub struct EventBroadcaster {
    sender: broadcast::Sender<UiEventEnvelope>,
}

impl EventBroadcaster {
    /// Create a new event broadcaster.
    #[must_use]
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(DEFAULT_CHANNEL_CAPACITY);
        Self { sender }
    }

    /// Create a new event broadcaster with custom capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Broadcast an event to all subscribers.
    ///
    /// Returns the number of subscribers that received the event.
    #[must_use]
    pub fn broadcast(&self, event: UiEvent) -> usize {
        let envelope = UiEventEnvelope::new(event);
        // Ignore send errors (no subscribers)
        self.sender.send(envelope).unwrap_or(0)
    }

    /// Subscribe to receive events.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<UiEventEnvelope> {
        self.sender.subscribe()
    }

    /// Get the number of active subscribers.
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventBroadcaster {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_envelope() {
        let event = UiEvent::Heartbeat {
            timestamp: Utc::now(),
        };
        let envelope = UiEventEnvelope::new(event);

        assert!(!envelope.id.is_empty());
        assert_eq!(envelope.id.len(), 16); // 8 bytes = 16 hex chars
    }

    #[tokio::test]
    async fn test_broadcaster() {
        let broadcaster = EventBroadcaster::new();
        let mut rx = broadcaster.subscribe();

        assert_eq!(broadcaster.subscriber_count(), 1);

        let event = UiEvent::Heartbeat {
            timestamp: Utc::now(),
        };
        let count = broadcaster.broadcast(event);
        assert_eq!(count, 1);

        let received = rx.recv().await.unwrap();
        match received.event {
            UiEvent::Heartbeat { .. } => {}
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_no_subscribers() {
        let broadcaster = EventBroadcaster::new();
        // Should not panic even with no subscribers
        let count = broadcaster.broadcast(UiEvent::Heartbeat {
            timestamp: Utc::now(),
        });
        assert_eq!(count, 0);
    }
}
