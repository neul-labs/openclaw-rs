//! Core types used throughout OpenClaw.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for an agent.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

impl AgentId {
    /// Create a new agent ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the default agent ID.
    #[must_use]
    pub fn default_agent() -> Self {
        Self("default".to_string())
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for AgentId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Unique identifier for a messaging channel.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChannelId(pub String);

impl ChannelId {
    /// Create a new channel ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Telegram channel.
    #[must_use]
    pub fn telegram() -> Self {
        Self("telegram".to_string())
    }

    /// Discord channel.
    #[must_use]
    pub fn discord() -> Self {
        Self("discord".to_string())
    }

    /// Slack channel.
    #[must_use]
    pub fn slack() -> Self {
        Self("slack".to_string())
    }

    /// WhatsApp channel.
    #[must_use]
    pub fn whatsapp() -> Self {
        Self("whatsapp".to_string())
    }

    /// Signal channel.
    #[must_use]
    pub fn signal() -> Self {
        Self("signal".to_string())
    }

    /// Matrix channel.
    #[must_use]
    pub fn matrix() -> Self {
        Self("matrix".to_string())
    }
}

impl fmt::Display for ChannelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for ChannelId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Unique identifier for a peer (user/contact) on a channel.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(pub String);

impl PeerId {
    /// Create a new peer ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for PeerId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Session key uniquely identifies a conversation session.
///
/// Format: `agent:<agent_id>:channel:<channel>:account:<account_id>:<peer_type>:<peer_id>`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionKey(pub String);

impl SessionKey {
    /// Create a new session key.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Build a session key from components.
    #[must_use]
    pub fn build(
        agent_id: &AgentId,
        channel: &ChannelId,
        account_id: &str,
        peer_type: PeerType,
        peer_id: &PeerId,
    ) -> Self {
        Self(format!(
            "agent:{}:channel:{}:account:{}:{}:{}",
            agent_id.0,
            channel.0,
            account_id,
            peer_type.as_str(),
            peer_id.0
        ))
    }

    /// Get the main session key for an agent.
    #[must_use]
    pub fn main_session(agent_id: &AgentId) -> Self {
        Self(format!("agent:{}:main", agent_id.0))
    }
}

impl fmt::Display for SessionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for SessionKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Type of peer conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PeerType {
    /// Direct message.
    Dm,
    /// Group chat.
    Group,
    /// Channel/broadcast.
    Channel,
    /// Thread within a group or channel.
    Thread,
}

impl PeerType {
    /// Get string representation.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dm => "dm",
            Self::Group => "group",
            Self::Channel => "channel",
            Self::Thread => "thread",
        }
    }
}

/// A normalized message from any channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID from the channel.
    pub id: String,
    /// Channel this message came from.
    pub channel: ChannelId,
    /// Account ID on the channel.
    pub account_id: String,
    /// Peer who sent the message.
    pub peer_id: PeerId,
    /// Type of peer conversation.
    pub peer_type: PeerType,
    /// Text content of the message.
    pub content: String,
    /// Attachments (media, files).
    pub attachments: Vec<Attachment>,
    /// Timestamp when the message was sent.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Reply-to message ID (if this is a reply).
    pub reply_to: Option<String>,
    /// Thread ID (if in a thread).
    pub thread_id: Option<String>,
    /// Mentioned user IDs.
    pub mentions: Vec<String>,
    /// Raw platform-specific data (for debugging).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

/// An attachment to a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Attachment type.
    pub kind: AttachmentKind,
    /// URL or path to the attachment.
    pub url: String,
    /// MIME type.
    pub mime_type: Option<String>,
    /// File name.
    pub filename: Option<String>,
    /// File size in bytes.
    pub size: Option<u64>,
    /// Thumbnail URL (for images/videos).
    pub thumbnail_url: Option<String>,
}

/// Type of attachment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentKind {
    /// Image file.
    Image,
    /// Video file.
    Video,
    /// Audio file.
    Audio,
    /// Voice message.
    Voice,
    /// Document/file.
    Document,
    /// Sticker.
    Sticker,
    /// GIF.
    Gif,
    /// Location.
    Location,
    /// Contact.
    Contact,
    /// Unknown type.
    Unknown,
}

/// Result of delivering an outbound message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryResult {
    /// Message ID assigned by the channel.
    pub message_id: String,
    /// Channel the message was sent to.
    pub channel: ChannelId,
    /// Timestamp when the message was delivered.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Chat/room ID.
    pub chat_id: Option<String>,
    /// Additional platform-specific metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/// Token usage statistics from an LLM call.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Input/prompt tokens.
    pub input_tokens: u64,
    /// Output/completion tokens.
    pub output_tokens: u64,
    /// Cache read tokens (if applicable).
    pub cache_read_tokens: Option<u64>,
    /// Cache write tokens (if applicable).
    pub cache_write_tokens: Option<u64>,
}

impl TokenUsage {
    /// Get total tokens used.
    #[must_use]
    pub fn total(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_key_build() {
        let key = SessionKey::build(
            &AgentId::new("default"),
            &ChannelId::telegram(),
            "bot123",
            PeerType::Dm,
            &PeerId::new("user456"),
        );
        assert_eq!(
            key.0,
            "agent:default:channel:telegram:account:bot123:dm:user456"
        );
    }

    #[test]
    fn test_channel_ids() {
        assert_eq!(ChannelId::telegram().0, "telegram");
        assert_eq!(ChannelId::discord().0, "discord");
        assert_eq!(ChannelId::slack().0, "slack");
    }
}
