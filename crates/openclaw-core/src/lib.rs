//! # `OpenClaw` Core
//!
//! Core types, configuration, and storage for `OpenClaw`.
//!
//! This crate provides:
//! - Configuration loading and validation (JSON5 format)
//! - Event-sourced session storage (grite pattern)
//! - CRDT projections for session state
//! - Secrets management with encryption at rest
//! - Input validation and sanitization

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod auth;
pub mod config;
pub mod events;
pub mod secrets;
pub mod types;
pub mod validation;

pub use auth::AuthProfile;
pub use config::{Config, ConfigError};
pub use events::{EventStore, SessionEvent, SessionEventKind, SessionProjection};
pub use secrets::CredentialStore;
pub use secrets::{ApiKey, scrub_secrets};
pub use types::{AgentId, ChannelId, Message, PeerId, SessionKey};
pub use validation::{ValidationError, validate_message_content};

/// Re-export commonly used external types
pub mod prelude {
    pub use crate::config::Config;
    pub use crate::events::{EventStore, SessionEvent, SessionProjection};
    pub use crate::secrets::ApiKey;
    pub use crate::types::*;
    pub use crate::validation::validate_message_content;
}
