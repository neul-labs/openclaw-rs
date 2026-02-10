//! Authentication middleware for axum.

use std::sync::Arc;

use axum::{
    Json,
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, header::AUTHORIZATION, request::Parts},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tokio::sync::RwLock;

use super::AuthError;
use super::config::AuthConfig;
use super::jwt::{Claims, JwtManager};
use super::setup::BootstrapManager;
use super::users::{UserRole, UserStore};

/// Shared authentication state.
pub struct AuthState {
    /// Auth configuration.
    pub config: AuthConfig,
    /// JWT manager.
    pub jwt: JwtManager,
    /// User store.
    pub users: UserStore,
    /// Bootstrap manager.
    pub bootstrap: RwLock<BootstrapManager>,
}

impl AuthState {
    /// Create a new auth state.
    #[must_use]
    pub fn new(config: AuthConfig, jwt: JwtManager, users: UserStore) -> Self {
        Self {
            config,
            jwt,
            users,
            bootstrap: RwLock::new(BootstrapManager::new()),
        }
    }

    /// Initialize auth state, auto-generating JWT secret if needed.
    ///
    /// # Errors
    ///
    /// Returns error if initialization fails.
    pub fn initialize(
        mut config: AuthConfig,
        data_dir: &std::path::Path,
    ) -> Result<Self, AuthError> {
        // Open user store
        let users = UserStore::open(data_dir)?;

        // Generate or load JWT secret
        let jwt_secret = match &config.jwt_secret {
            Some(secret) => secret.clone(),
            None => {
                let secret = JwtManager::generate_hex_secret();
                config.jwt_secret = Some(secret.clone());
                // In a real implementation, we'd persist this to config
                tracing::info!("Generated new JWT secret");
                secret
            }
        };

        let jwt = JwtManager::from_hex_secret(
            &jwt_secret,
            config.token_expiry(),
            config.refresh_expiry(),
        )?;

        Ok(Self::new(config, jwt, users))
    }

    /// Check if auth is required for a method.
    #[must_use]
    pub fn requires_auth(&self, method: &str) -> bool {
        self.config.enabled && !self.config.is_public_method(method)
    }

    /// Validate a token and return claims.
    ///
    /// # Errors
    ///
    /// Returns error if token is invalid.
    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        self.jwt.validate_access_token(token)
    }
}

impl std::fmt::Debug for AuthState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthState")
            .field("config", &self.config)
            .field("user_count", &self.users.count())
            .finish_non_exhaustive()
    }
}

/// Auth layer marker for protected routes.
#[derive(Debug, Clone)]
pub struct AuthLayer;

/// Extractor for authenticated requests.
///
/// Use this in handler parameters to require authentication.
#[derive(Debug, Clone)]
pub struct RequireAuth {
    /// The authenticated user's claims.
    pub claims: Claims,
}

impl RequireAuth {
    /// Get the user ID.
    #[must_use]
    pub fn user_id(&self) -> &str {
        &self.claims.sub
    }

    /// Get the username.
    #[must_use]
    pub fn username(&self) -> &str {
        &self.claims.username
    }

    /// Get the user role.
    #[must_use]
    pub fn role(&self) -> UserRole {
        self.claims.role
    }

    /// Check if user is admin.
    #[must_use]
    pub fn is_admin(&self) -> bool {
        self.claims.role.is_admin()
    }

    /// Require admin role.
    ///
    /// # Errors
    ///
    /// Returns error if user is not admin.
    pub fn require_admin(&self) -> Result<(), AuthError> {
        if self.is_admin() {
            Ok(())
        } else {
            Err(AuthError::PermissionDenied(
                "Admin role required".to_string(),
            ))
        }
    }
}

/// Error response for auth failures.
#[derive(Debug, Serialize)]
struct AuthErrorResponse {
    error: String,
    code: &'static str,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "invalid_credentials"),
            AuthError::TokenError(_) => (StatusCode::UNAUTHORIZED, "invalid_token"),
            AuthError::PermissionDenied(_) => (StatusCode::FORBIDDEN, "permission_denied"),
            AuthError::SetupRequired => (StatusCode::SERVICE_UNAVAILABLE, "setup_required"),
            AuthError::InvalidBootstrapToken => {
                (StatusCode::UNAUTHORIZED, "invalid_bootstrap_token")
            }
            AuthError::UserNotFound(_) => (StatusCode::NOT_FOUND, "user_not_found"),
            AuthError::UserExists(_) => (StatusCode::CONFLICT, "user_exists"),
            AuthError::Storage(_) | AuthError::Config(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
            }
        };

        let body = AuthErrorResponse {
            error: self.to_string(),
            code,
        };

        (status, Json(body)).into_response()
    }
}

/// Extractor implementation for `RequireAuth`.
impl<S> FromRequestParts<S> for RequireAuth
where
    S: Send + Sync,
    Arc<AuthState>: FromRef<S>,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_state = Arc::<AuthState>::from_ref(state);
        extract_auth(parts, &auth_state).await
    }
}

async fn extract_auth(parts: &Parts, auth_state: &AuthState) -> Result<RequireAuth, Response> {
    // Check if auth is disabled
    if !auth_state.config.enabled {
        // Return a dummy admin claim when auth is disabled
        return Ok(RequireAuth {
            claims: Claims {
                sub: "system".to_string(),
                username: "system".to_string(),
                role: UserRole::Admin,
                iat: 0,
                exp: i64::MAX,
                token_type: super::jwt::TokenType::Access,
                family_id: None,
            },
        });
    }

    // Extract token from Authorization header
    let auth_header = parts
        .headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            AuthError::TokenError("Missing Authorization header".to_string()).into_response()
        })?;

    let token = JwtManager::extract_from_header(auth_header).ok_or_else(|| {
        AuthError::TokenError("Invalid Authorization header format".to_string()).into_response()
    })?;

    // Validate token
    let claims = auth_state
        .validate_token(token)
        .map_err(IntoResponse::into_response)?;

    // Check if user is still active
    let user = auth_state
        .users
        .get(&claims.sub)
        .map_err(IntoResponse::into_response)?
        .ok_or_else(|| AuthError::UserNotFound(claims.sub.clone()).into_response())?;

    if !user.active {
        return Err(AuthError::PermissionDenied("Account disabled".to_string()).into_response());
    }

    Ok(RequireAuth { claims })
}

/// Extractor for optional authentication.
///
/// Returns `None` if no valid auth is present, `Some(RequireAuth)` otherwise.
#[derive(Debug, Clone)]
pub struct OptionalAuth(pub Option<RequireAuth>);

impl<S> FromRequestParts<S> for OptionalAuth
where
    S: Send + Sync,
    Arc<AuthState>: FromRef<S>,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(OptionalAuth(
            RequireAuth::from_request_parts(parts, state).await.ok(),
        ))
    }
}

/// Require admin role extractor.
#[derive(Debug, Clone)]
pub struct RequireAdmin(pub RequireAuth);

impl<S> FromRequestParts<S> for RequireAdmin
where
    S: Send + Sync,
    Arc<AuthState>: FromRef<S>,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = RequireAuth::from_request_parts(parts, state).await?;

        if !auth.is_admin() {
            return Err(
                AuthError::PermissionDenied("Admin role required".to_string()).into_response(),
            );
        }

        Ok(RequireAdmin(auth))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_require_auth_methods() {
        let auth = RequireAuth {
            claims: Claims {
                sub: "user_123".to_string(),
                username: "testuser".to_string(),
                role: UserRole::Operator,
                iat: 0,
                exp: i64::MAX,
                token_type: super::super::jwt::TokenType::Access,
                family_id: None,
            },
        };

        assert_eq!(auth.user_id(), "user_123");
        assert_eq!(auth.username(), "testuser");
        assert_eq!(auth.role(), UserRole::Operator);
        assert!(!auth.is_admin());
        assert!(auth.require_admin().is_err());
    }

    #[test]
    fn test_admin_auth() {
        let auth = RequireAuth {
            claims: Claims {
                sub: "admin_1".to_string(),
                username: "admin".to_string(),
                role: UserRole::Admin,
                iat: 0,
                exp: i64::MAX,
                token_type: super::super::jwt::TokenType::Access,
                family_id: None,
            },
        };

        assert!(auth.is_admin());
        assert!(auth.require_admin().is_ok());
    }
}
