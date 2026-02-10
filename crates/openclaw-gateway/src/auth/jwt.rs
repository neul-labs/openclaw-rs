//! JWT token management.

use std::time::Duration;

use chrono::{DateTime, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode};
use rand::RngCore;
use serde::{Deserialize, Serialize};

use super::AuthError;
use super::users::UserRole;

/// JWT claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID).
    pub sub: String,
    /// Username.
    pub username: String,
    /// User role.
    pub role: UserRole,
    /// Issued at (Unix timestamp).
    pub iat: i64,
    /// Expiration (Unix timestamp).
    pub exp: i64,
    /// Token type (access or refresh).
    #[serde(default)]
    pub token_type: TokenType,
    /// Token family ID (for refresh token rotation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_id: Option<String>,
}

/// Token type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    /// Access token for API calls.
    #[default]
    Access,
    /// Refresh token for getting new access tokens.
    Refresh,
}

/// A pair of access and refresh tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    /// Access token.
    pub access_token: String,
    /// Refresh token.
    pub refresh_token: String,
    /// Access token expiration.
    pub expires_at: DateTime<Utc>,
    /// Refresh token expiration.
    pub refresh_expires_at: DateTime<Utc>,
    /// Token type (always "Bearer").
    pub token_type: String,
}

/// JWT manager for creating and validating tokens.
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_expiry: Duration,
    refresh_expiry: Duration,
}

impl JwtManager {
    /// Create a new JWT manager with a secret key.
    ///
    /// The secret should be at least 32 bytes for security.
    #[must_use]
    pub fn new(secret: &[u8], access_expiry: Duration, refresh_expiry: Duration) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            access_expiry,
            refresh_expiry,
        }
    }

    /// Create a JWT manager from a hex-encoded secret.
    ///
    /// # Errors
    ///
    /// Returns error if hex decoding fails.
    pub fn from_hex_secret(
        hex_secret: &str,
        access_expiry: Duration,
        refresh_expiry: Duration,
    ) -> Result<Self, AuthError> {
        let secret = hex::decode(hex_secret)
            .map_err(|e| AuthError::Config(format!("Invalid hex secret: {e}")))?;
        Ok(Self::new(&secret, access_expiry, refresh_expiry))
    }

    /// Generate a random 256-bit secret key.
    #[must_use]
    pub fn generate_secret() -> [u8; 32] {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        bytes
    }

    /// Generate a random secret as hex string.
    #[must_use]
    pub fn generate_hex_secret() -> String {
        hex::encode(Self::generate_secret())
    }

    /// Create an access token for a user.
    ///
    /// # Errors
    ///
    /// Returns error if token encoding fails.
    pub fn create_access_token(
        &self,
        user_id: &str,
        username: &str,
        role: UserRole,
    ) -> Result<(String, DateTime<Utc>), AuthError> {
        let now = Utc::now();
        let exp = now + chrono::Duration::from_std(self.access_expiry).unwrap_or_default();

        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            role,
            iat: now.timestamp(),
            exp: exp.timestamp(),
            token_type: TokenType::Access,
            family_id: None,
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AuthError::TokenError(format!("Encoding failed: {e}")))?;

        Ok((token, exp))
    }

    /// Create a refresh token for a user.
    ///
    /// # Errors
    ///
    /// Returns error if token encoding fails.
    pub fn create_refresh_token(
        &self,
        user_id: &str,
        username: &str,
        role: UserRole,
        family_id: Option<String>,
    ) -> Result<(String, DateTime<Utc>), AuthError> {
        let now = Utc::now();
        let exp = now + chrono::Duration::from_std(self.refresh_expiry).unwrap_or_default();

        // Generate new family ID if not provided (new login)
        let family_id = family_id.unwrap_or_else(|| {
            let mut bytes = [0u8; 16];
            rand::thread_rng().fill_bytes(&mut bytes);
            hex::encode(bytes)
        });

        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            role,
            iat: now.timestamp(),
            exp: exp.timestamp(),
            token_type: TokenType::Refresh,
            family_id: Some(family_id),
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AuthError::TokenError(format!("Encoding failed: {e}")))?;

        Ok((token, exp))
    }

    /// Create a token pair (access + refresh) for a user.
    ///
    /// # Errors
    ///
    /// Returns error if token creation fails.
    pub fn create_token_pair(
        &self,
        user_id: &str,
        username: &str,
        role: UserRole,
    ) -> Result<TokenPair, AuthError> {
        let (access_token, expires_at) = self.create_access_token(user_id, username, role)?;
        let (refresh_token, refresh_expires_at) =
            self.create_refresh_token(user_id, username, role, None)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_at,
            refresh_expires_at,
            token_type: "Bearer".to_string(),
        })
    }

    /// Validate and decode a token.
    ///
    /// # Errors
    ///
    /// Returns error if token is invalid or expired.
    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        let validation = Validation::default();

        let token_data: TokenData<Claims> = decode(token, &self.decoding_key, &validation)
            .map_err(|e| AuthError::TokenError(format!("Validation failed: {e}")))?;

        Ok(token_data.claims)
    }

    /// Validate an access token (must be access type).
    ///
    /// # Errors
    ///
    /// Returns error if token is invalid, expired, or not an access token.
    pub fn validate_access_token(&self, token: &str) -> Result<Claims, AuthError> {
        let claims = self.validate_token(token)?;

        if claims.token_type != TokenType::Access {
            return Err(AuthError::TokenError("Not an access token".to_string()));
        }

        Ok(claims)
    }

    /// Validate a refresh token and optionally create new tokens.
    ///
    /// # Errors
    ///
    /// Returns error if token is invalid, expired, or not a refresh token.
    pub fn refresh_tokens(&self, refresh_token: &str) -> Result<TokenPair, AuthError> {
        let claims = self.validate_token(refresh_token)?;

        if claims.token_type != TokenType::Refresh {
            return Err(AuthError::TokenError("Not a refresh token".to_string()));
        }

        // Create new tokens with the same family ID (for rotation tracking)
        let (access_token, expires_at) =
            self.create_access_token(&claims.sub, &claims.username, claims.role)?;
        let (new_refresh_token, refresh_expires_at) = self.create_refresh_token(
            &claims.sub,
            &claims.username,
            claims.role,
            claims.family_id,
        )?;

        Ok(TokenPair {
            access_token,
            refresh_token: new_refresh_token,
            expires_at,
            refresh_expires_at,
            token_type: "Bearer".to_string(),
        })
    }

    /// Extract token from Authorization header.
    ///
    /// Expects format: "Bearer <token>"
    #[must_use]
    pub fn extract_from_header(header: &str) -> Option<&str> {
        header
            .strip_prefix("Bearer ")
            .or_else(|| header.strip_prefix("bearer "))
    }
}

impl std::fmt::Debug for JwtManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtManager")
            .field("access_expiry", &self.access_expiry)
            .field("refresh_expiry", &self.refresh_expiry)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_manager() -> JwtManager {
        let secret = JwtManager::generate_secret();
        JwtManager::new(
            &secret,
            Duration::from_secs(3600),      // 1 hour
            Duration::from_secs(7 * 86400), // 7 days
        )
    }

    #[test]
    fn test_generate_secret() {
        let secret1 = JwtManager::generate_secret();
        let secret2 = JwtManager::generate_secret();
        assert_ne!(secret1, secret2);
        assert_eq!(secret1.len(), 32);
    }

    #[test]
    fn test_create_access_token() {
        let manager = create_manager();
        let (token, expires) = manager
            .create_access_token("user_123", "testuser", UserRole::Admin)
            .unwrap();

        assert!(!token.is_empty());
        assert!(expires > Utc::now());
    }

    #[test]
    fn test_validate_token() {
        let manager = create_manager();
        let (token, _) = manager
            .create_access_token("user_123", "testuser", UserRole::Operator)
            .unwrap();

        let claims = manager.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "user_123");
        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.role, UserRole::Operator);
        assert_eq!(claims.token_type, TokenType::Access);
    }

    #[test]
    fn test_token_pair() {
        let manager = create_manager();
        let pair = manager
            .create_token_pair("user_123", "admin", UserRole::Admin)
            .unwrap();

        assert!(!pair.access_token.is_empty());
        assert!(!pair.refresh_token.is_empty());
        assert_eq!(pair.token_type, "Bearer");

        // Validate access token
        let access_claims = manager.validate_access_token(&pair.access_token).unwrap();
        assert_eq!(access_claims.token_type, TokenType::Access);

        // Validate refresh token
        let refresh_claims = manager.validate_token(&pair.refresh_token).unwrap();
        assert_eq!(refresh_claims.token_type, TokenType::Refresh);
    }

    #[test]
    fn test_refresh_tokens() {
        let manager = create_manager();
        let pair = manager
            .create_token_pair("user_123", "admin", UserRole::Admin)
            .unwrap();

        // Refresh tokens should produce valid new tokens
        let new_pair = manager.refresh_tokens(&pair.refresh_token).unwrap();

        // Verify new tokens are valid
        let access_claims = manager
            .validate_access_token(&new_pair.access_token)
            .unwrap();
        assert_eq!(access_claims.sub, "user_123");
        assert_eq!(access_claims.username, "admin");
        assert_eq!(access_claims.role, UserRole::Admin);

        let refresh_claims = manager.validate_token(&new_pair.refresh_token).unwrap();
        assert_eq!(refresh_claims.token_type, TokenType::Refresh);

        // The new refresh token should also be valid for refreshing
        let third_pair = manager.refresh_tokens(&new_pair.refresh_token).unwrap();
        assert!(
            manager
                .validate_access_token(&third_pair.access_token)
                .is_ok()
        );
    }

    #[test]
    fn test_invalid_token() {
        let manager = create_manager();
        let result = manager.validate_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_from_header() {
        assert_eq!(
            JwtManager::extract_from_header("Bearer abc123"),
            Some("abc123")
        );
        assert_eq!(
            JwtManager::extract_from_header("bearer abc123"),
            Some("abc123")
        );
        assert_eq!(JwtManager::extract_from_header("abc123"), None);
    }

    #[test]
    fn test_hex_secret() {
        let hex_secret = JwtManager::generate_hex_secret();
        assert_eq!(hex_secret.len(), 64); // 32 bytes = 64 hex chars

        let manager = JwtManager::from_hex_secret(
            &hex_secret,
            Duration::from_secs(3600),
            Duration::from_secs(86400),
        )
        .unwrap();

        let (token, _) = manager
            .create_access_token("user_123", "test", UserRole::Viewer)
            .unwrap();
        assert!(manager.validate_token(&token).is_ok());
    }
}
