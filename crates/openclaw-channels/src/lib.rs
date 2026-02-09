//! # OpenClaw Channels
//!
//! Channel adapters for messaging platforms.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod traits;
mod routing;
mod allowlist;
mod registry;

/// Telegram channel adapter.
pub mod telegram;
/// Discord channel adapter.
pub mod discord;
/// Slack channel adapter.
pub mod slack;
/// Signal channel adapter.
pub mod signal;
/// Matrix channel adapter.
pub mod matrix;
/// WhatsApp channel adapter.
pub mod whatsapp;

pub use traits::{
    Channel, ChannelInbound, ChannelOutbound, ChannelError, ChannelProbe,
    ChannelCapabilities, ChannelContext, OutboundContext, DeliveryMode,
};
pub use routing::AgentRouter;
pub use allowlist::{Allowlist, AllowlistEntry};
pub use registry::ChannelRegistry;

// Re-export channel implementations
pub use telegram::TelegramChannel;
pub use discord::DiscordChannel;
pub use slack::SlackChannel;
pub use signal::SignalChannel;
pub use matrix::MatrixChannel;
pub use whatsapp::WhatsAppChannel;
