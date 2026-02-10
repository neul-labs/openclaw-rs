//! Channel traits.

use async_trait::async_trait;
use thiserror::Error;

use openclaw_core::types::{Attachment, DeliveryResult, Message};

/// Channel errors.
#[derive(Error, Debug)]
pub enum ChannelError {
    /// Channel not connected.
    #[error("Channel not connected")]
    NotConnected,

    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    /// Message delivery failed.
    #[error("Delivery failed: {0}")]
    DeliveryFailed(String),

    /// Rate limited.
    #[error("Rate limited")]
    RateLimited,

    /// Network error.
    #[error("Network error: {0}")]
    Network(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Channel capabilities.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ChannelCapabilities {
    /// Supports text messages.
    pub text: bool,
    /// Supports images.
    pub images: bool,
    /// Supports videos.
    pub videos: bool,
    /// Supports voice messages.
    pub voice: bool,
    /// Supports files.
    pub files: bool,
    /// Supports threads.
    pub threads: bool,
    /// Supports reactions.
    pub reactions: bool,
    /// Supports editing messages.
    pub editing: bool,
    /// Supports deleting messages.
    pub deletion: bool,
}

/// Channel health probe result.
#[derive(Debug, Clone)]
pub struct ChannelProbe {
    /// Whether channel is connected.
    pub connected: bool,
    /// Account/bot identifier.
    pub account_id: Option<String>,
    /// Account display name.
    pub display_name: Option<String>,
    /// Error message if not connected.
    pub error: Option<String>,
}

/// Delivery mode for outbound messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryMode {
    /// Send immediately.
    Immediate,
    /// Queue for batched delivery.
    Batched,
    /// Webhook-based delivery.
    Webhook,
}

/// Context for outbound messages.
#[derive(Debug, Clone)]
pub struct OutboundContext {
    /// Target chat/channel ID.
    pub chat_id: String,
    /// Reply to message ID.
    pub reply_to: Option<String>,
    /// Thread ID.
    pub thread_id: Option<String>,
}

/// Context for channel operations.
#[derive(Debug, Clone)]
pub struct ChannelContext {
    /// Agent ID for routing.
    pub agent_id: String,
    /// Account ID on the channel.
    pub account_id: String,
}

/// Core channel trait.
#[async_trait]
pub trait Channel: Send + Sync {
    /// Channel identifier (e.g., "telegram", "discord").
    fn id(&self) -> &str;

    /// Human-readable label.
    fn label(&self) -> &str;

    /// Channel capabilities.
    fn capabilities(&self) -> ChannelCapabilities;

    /// Start monitoring for inbound messages.
    async fn start(&self, ctx: ChannelContext) -> Result<(), ChannelError>;

    /// Stop monitoring.
    async fn stop(&self) -> Result<(), ChannelError>;

    /// Check if channel is ready.
    async fn probe(&self) -> Result<ChannelProbe, ChannelError>;
}

/// Outbound message delivery trait.
#[async_trait]
pub trait ChannelOutbound: Channel {
    /// Send a text message.
    async fn send_text(
        &self,
        ctx: OutboundContext,
        text: &str,
    ) -> Result<DeliveryResult, ChannelError>;

    /// Send media attachments.
    async fn send_media(
        &self,
        ctx: OutboundContext,
        media: &[Attachment],
    ) -> Result<DeliveryResult, ChannelError>;

    /// Maximum text message length.
    fn text_chunk_limit(&self) -> usize;

    /// Delivery mode.
    fn delivery_mode(&self) -> DeliveryMode;
}

/// Inbound message handling trait.
#[async_trait]
pub trait ChannelInbound: Channel {
    /// Raw message type from the platform.
    type RawMessage;

    /// Normalize raw message to common format.
    fn normalize(&self, raw: Self::RawMessage) -> Result<Message, ChannelError>;

    /// Acknowledge message receipt.
    async fn acknowledge(&self, message_id: &str) -> Result<(), ChannelError>;
}
