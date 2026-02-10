//! # OpenClaw Gateway
//!
//! HTTP/WebSocket gateway server with JSON-RPC protocol.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Authentication and authorization.
pub mod auth;
/// WebSocket UI events.
pub mod events;
mod middleware;
/// JSON-RPC protocol types and constants.
pub mod rpc;
mod server;

/// UI static file server (requires "ui" feature).
#[cfg(feature = "ui")]
pub mod ui_server;

pub use auth::{AuthConfig, AuthError, AuthState, User, UserRole, UserStore};
pub use events::{EventBroadcaster, UiEvent, UiEventEnvelope};
pub use middleware::GatewayRateLimiter;
pub use rpc::{RpcError, RpcRequest, RpcResponse};
pub use server::{Gateway, GatewayBuilder, GatewayConfig, GatewayState};

#[cfg(feature = "ui")]
pub use ui_server::UiServerConfig;

/// Start the gateway server.
///
/// # Errors
///
/// Returns error if server fails to start.
pub async fn start(config: GatewayConfig) -> Result<(), GatewayError> {
    let gateway = Gateway::new(config)?;
    gateway.run().await
}

/// Gateway errors.
#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    /// Server error.
    #[error("Server error: {0}")]
    Server(String),

    /// Configuration error.
    #[error("Config error: {0}")]
    Config(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
