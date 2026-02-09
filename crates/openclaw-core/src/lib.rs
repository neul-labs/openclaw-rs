//! # OpenClaw Core
//!
//! Core types, configuration, and storage for OpenClaw.
//!
//! This crate provides:
//! - Configuration loading and validation (JSON5 format)
//! - Event-sourced session storage (grite pattern)
//! - CRDT projections for session state
//! - Secrets management with encryption at rest
//! - Input validation and sanitization

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod config;
pub mod events;
pub mod auth;
pub mod types;
pub mod validation;
pub mod secrets;

pub use config::{Config, ConfigError};
pub use events::{EventStore, SessionEvent, SessionEventKind, SessionProjection};
pub use auth::AuthProfile;
pub use secrets::CredentialStore;
pub use types::{AgentId, ChannelId, SessionKey, Message, PeerId};
pub use validation::{ValidationError, validate_message_content};
pub use secrets::{ApiKey, scrub_secrets};

/// Re-export commonly used external types
pub mod prelude {
    pub use crate::config::Config;
    pub use crate::events::{EventStore, SessionEvent, SessionProjection};
    pub use crate::types::*;
    pub use crate::validation::validate_message_content;
    pub use crate::secrets::ApiKey;
}
