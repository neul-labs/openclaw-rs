//! # OpenClaw Channels
//!
//! Channel adapters for messaging platforms.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod allowlist;
mod registry;
mod routing;
mod traits;

/// Discord channel adapter.
pub mod discord;
/// Matrix channel adapter.
pub mod matrix;
/// Signal channel adapter.
pub mod signal;
/// Slack channel adapter.
pub mod slack;
/// Telegram channel adapter.
pub mod telegram;
/// WhatsApp channel adapter.
pub mod whatsapp;

pub use allowlist::{Allowlist, AllowlistEntry};
pub use registry::ChannelRegistry;
pub use routing::AgentRouter;
pub use traits::{
    Channel, ChannelCapabilities, ChannelContext, ChannelError, ChannelInbound, ChannelOutbound,
    ChannelProbe, DeliveryMode, OutboundContext,
};

// Re-export channel implementations
pub use discord::DiscordChannel;
pub use matrix::MatrixChannel;
pub use signal::SignalChannel;
pub use slack::SlackChannel;
pub use telegram::TelegramChannel;
pub use whatsapp::WhatsAppChannel;
