//! Authentication and authorization for the gateway.
//!
//! This module provides:
//! - User management with role-based access control
//! - JWT token generation and validation
//! - First-run setup and bootstrap
//! - Auth middleware for protected routes

mod config;
mod jwt;
mod middleware;
/// First-run setup and bootstrap management.
pub mod setup;
mod users;

pub use config::{AuthConfig, AuthConfigBuilder};
pub use jwt::{Claims, JwtManager, TokenPair};
pub use middleware::{AuthLayer, AuthState, RequireAuth};
pub use setup::{BootstrapManager, SetupStatus};
pub use users::{User, UserRole, UserStore};

use thiserror::Error;

/// Authentication errors.
#[derive(Debug, Error)]
pub enum AuthError {
    /// Invalid credentials provided.
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// User not found.
    #[error("User not found: {0}")]
    UserNotFound(String),

    /// User already exists.
    #[error("User already exists: {0}")]
    UserExists(String),

    /// Token error (expired, invalid, etc.).
    #[error("Token error: {0}")]
    TokenError(String),

    /// Permission denied.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Setup required (no users exist).
    #[error("Setup required: no admin user configured")]
    SetupRequired,

    /// Bootstrap token invalid or expired.
    #[error("Invalid or expired bootstrap token")]
    InvalidBootstrapToken,

    /// Storage error.
    #[error("Storage error: {0}")]
    Storage(String),

    /// Configuration error.
    #[error("Config error: {0}")]
    Config(String),
}
