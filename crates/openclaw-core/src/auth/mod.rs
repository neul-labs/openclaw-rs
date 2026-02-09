//! Authentication profiles and credential management.
//!
//! Manages OAuth tokens, API keys, and session credentials
//! for channels and providers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

use crate::secrets::{ApiKey, CredentialError, CredentialStore};

/// Authentication errors.
#[derive(Error, Debug)]
pub enum AuthError {
    /// Credential error.
    #[error("Credential error: {0}")]
    Credential(#[from] CredentialError),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Token expired.
    #[error("Token expired")]
    TokenExpired,

    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    /// Profile not found.
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),
}

/// Authentication profile for a channel or provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthProfile {
    /// Profile identifier.
    pub id: String,

    /// Profile type.
    pub profile_type: ProfileType,

    /// Channel or provider this profile is for.
    pub target: String,

    /// Account identifier (e.g., bot username, phone number).
    pub account_id: Option<String>,

    /// When the profile was created.
    pub created_at: DateTime<Utc>,

    /// When the profile was last used.
    pub last_used: Option<DateTime<Utc>>,

    /// Whether this profile is active.
    pub active: bool,

    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AuthProfile {
    /// Create a new auth profile.
    #[must_use]
    pub fn new(id: impl Into<String>, profile_type: ProfileType, target: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            profile_type,
            target: target.into(),
            account_id: None,
            created_at: Utc::now(),
            last_used: None,
            active: true,
            metadata: HashMap::new(),
        }
    }

    /// Mark the profile as used.
    pub fn mark_used(&mut self) {
        self.last_used = Some(Utc::now());
    }
}

/// Type of authentication profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileType {
    /// API key authentication.
    ApiKey,
    /// OAuth 2.0 token.
    OAuth,
    /// Bot token (e.g., Telegram, Discord).
    BotToken,
    /// Session-based auth (e.g., WhatsApp, Signal).
    Session,
    /// Certificate-based auth.
    Certificate,
}

/// OAuth token data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    /// Access token.
    pub access_token: String,

    /// Refresh token (if available).
    pub refresh_token: Option<String>,

    /// Token expiration time.
    pub expires_at: Option<DateTime<Utc>>,

    /// Token type (usually "Bearer").
    pub token_type: String,

    /// Scopes granted.
    pub scopes: Vec<String>,
}

impl OAuthToken {
    /// Check if the token is expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|exp| exp < Utc::now())
            .unwrap_or(false)
    }

    /// Check if the token needs refresh (expires within 5 minutes).
    #[must_use]
    pub fn needs_refresh(&self) -> bool {
        self.expires_at
            .map(|exp| exp < Utc::now() + chrono::Duration::minutes(5))
            .unwrap_or(false)
    }
}

/// Credential store for authentication data.
///
/// Wraps `CredentialStore` with profile management.
pub struct AuthCredentialStore {
    inner: CredentialStore,
    profiles_path: std::path::PathBuf,
    profiles: HashMap<String, AuthProfile>,
}

impl AuthCredentialStore {
    /// Create a new auth credential store.
    ///
    /// # Arguments
    ///
    /// * `encryption_key` - 32-byte encryption key
    /// * `base_path` - Base directory for credentials
    ///
    /// # Errors
    ///
    /// Returns error if profile loading fails.
    pub fn new(encryption_key: [u8; 32], base_path: &Path) -> Result<Self, AuthError> {
        let store_path = base_path.join("secrets");
        let profiles_path = base_path.join("profiles.json");

        let inner = CredentialStore::new(encryption_key, store_path);

        let profiles = if profiles_path.exists() {
            let content = std::fs::read_to_string(&profiles_path)?;
            serde_json::from_str(&content)?
        } else {
            HashMap::new()
        };

        Ok(Self {
            inner,
            profiles_path,
            profiles,
        })
    }

    /// Store an API key for a profile.
    ///
    /// # Errors
    ///
    /// Returns error if storage fails.
    pub fn store_api_key(&mut self, profile_id: &str, key: &ApiKey) -> Result<(), AuthError> {
        self.inner.store(profile_id, key)?;
        self.save_profiles()?;
        Ok(())
    }

    /// Load an API key for a profile.
    ///
    /// # Errors
    ///
    /// Returns error if profile not found or decryption fails.
    pub fn load_api_key(&self, profile_id: &str) -> Result<ApiKey, AuthError> {
        Ok(self.inner.load(profile_id)?)
    }

    /// Store an OAuth token for a profile.
    ///
    /// # Errors
    ///
    /// Returns error if storage fails.
    pub fn store_oauth_token(&mut self, profile_id: &str, token: &OAuthToken) -> Result<(), AuthError> {
        let token_json = serde_json::to_string(token)?;
        let key = ApiKey::new(token_json);
        self.inner.store(&format!("{profile_id}_oauth"), &key)?;
        self.save_profiles()?;
        Ok(())
    }

    /// Load an OAuth token for a profile.
    ///
    /// # Errors
    ///
    /// Returns error if profile not found, decryption fails, or token expired.
    pub fn load_oauth_token(&self, profile_id: &str) -> Result<OAuthToken, AuthError> {
        let key = self.inner.load(&format!("{profile_id}_oauth"))?;
        let token: OAuthToken = serde_json::from_str(key.expose())?;

        if token.is_expired() && token.refresh_token.is_none() {
            return Err(AuthError::TokenExpired);
        }

        Ok(token)
    }

    /// Add or update an auth profile.
    pub fn set_profile(&mut self, profile: AuthProfile) -> Result<(), AuthError> {
        self.profiles.insert(profile.id.clone(), profile);
        self.save_profiles()?;
        Ok(())
    }

    /// Get an auth profile.
    #[must_use]
    pub fn get_profile(&self, profile_id: &str) -> Option<&AuthProfile> {
        self.profiles.get(profile_id)
    }

    /// Get a mutable auth profile.
    #[must_use]
    pub fn get_profile_mut(&mut self, profile_id: &str) -> Option<&mut AuthProfile> {
        self.profiles.get_mut(profile_id)
    }

    /// Remove an auth profile and its credentials.
    ///
    /// # Errors
    ///
    /// Returns error if deletion fails.
    pub fn remove_profile(&mut self, profile_id: &str) -> Result<(), AuthError> {
        self.profiles.remove(profile_id);
        let _ = self.inner.delete(profile_id);
        let _ = self.inner.delete(&format!("{profile_id}_oauth"));
        self.save_profiles()?;
        Ok(())
    }

    /// List all profiles.
    #[must_use]
    pub fn list_profiles(&self) -> Vec<&AuthProfile> {
        self.profiles.values().collect()
    }

    /// List profiles for a specific target (channel or provider).
    #[must_use]
    pub fn profiles_for_target(&self, target: &str) -> Vec<&AuthProfile> {
        self.profiles
            .values()
            .filter(|p| p.target == target)
            .collect()
    }

    /// Get the active profile for a target.
    #[must_use]
    pub fn active_profile_for_target(&self, target: &str) -> Option<&AuthProfile> {
        self.profiles
            .values()
            .find(|p| p.target == target && p.active)
    }

    /// Save profiles to disk.
    fn save_profiles(&self) -> Result<(), AuthError> {
        if let Some(parent) = self.profiles_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.profiles)?;
        std::fs::write(&self.profiles_path, content)?;
        Ok(())
    }
}

/// Refresh an OAuth token using the refresh token.
///
/// This is a placeholder - actual implementation depends on the OAuth provider.
pub async fn refresh_oauth_token(
    token: &OAuthToken,
    client_id: &str,
    client_secret: &str,
    token_url: &str,
) -> Result<OAuthToken, AuthError> {
    let refresh_token = token
        .refresh_token
        .as_ref()
        .ok_or_else(|| AuthError::AuthFailed("No refresh token available".to_string()))?;

    // Build refresh request
    let client = reqwest::Client::new();
    let response = client
        .post(token_url)
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", client_id),
            ("client_secret", client_secret),
        ])
        .send()
        .await
        .map_err(|e| AuthError::AuthFailed(e.to_string()))?;

    if !response.status().is_success() {
        return Err(AuthError::AuthFailed(format!(
            "Token refresh failed: {}",
            response.status()
        )));
    }

    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
        refresh_token: Option<String>,
        expires_in: Option<i64>,
        token_type: Option<String>,
    }

    let token_response: TokenResponse = response
        .json()
        .await
        .map_err(|e| AuthError::AuthFailed(e.to_string()))?;

    let expires_at = token_response
        .expires_in
        .map(|secs| Utc::now() + chrono::Duration::seconds(secs));

    Ok(OAuthToken {
        access_token: token_response.access_token,
        refresh_token: token_response
            .refresh_token
            .or_else(|| token.refresh_token.clone()),
        expires_at,
        token_type: token_response
            .token_type
            .unwrap_or_else(|| "Bearer".to_string()),
        scopes: token.scopes.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_auth_profile_creation() {
        let profile = AuthProfile::new("test-profile", ProfileType::ApiKey, "anthropic");

        assert_eq!(profile.id, "test-profile");
        assert_eq!(profile.target, "anthropic");
        assert!(profile.active);
    }

    #[test]
    fn test_oauth_token_expiry() {
        let token = OAuthToken {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: Some(Utc::now() - chrono::Duration::hours(1)),
            token_type: "Bearer".to_string(),
            scopes: vec![],
        };

        assert!(token.is_expired());
        assert!(token.needs_refresh());
    }

    #[test]
    fn test_oauth_token_valid() {
        let token = OAuthToken {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
            token_type: "Bearer".to_string(),
            scopes: vec![],
        };

        assert!(!token.is_expired());
        assert!(!token.needs_refresh());
    }

    #[test]
    fn test_auth_credential_store() {
        let temp = tempdir().unwrap();
        let encryption_key: [u8; 32] = rand::random();

        let mut store = AuthCredentialStore::new(encryption_key, temp.path()).unwrap();

        // Create and store a profile
        let profile = AuthProfile::new("test", ProfileType::ApiKey, "anthropic");
        store.set_profile(profile).unwrap();

        // Store API key
        let key = ApiKey::new("sk-test-key".to_string());
        store.store_api_key("test", &key).unwrap();

        // Retrieve
        let loaded = store.load_api_key("test").unwrap();
        assert_eq!(loaded.expose(), "sk-test-key");

        // List profiles
        let profiles = store.list_profiles();
        assert_eq!(profiles.len(), 1);
    }

    #[test]
    fn test_profiles_for_target() {
        let temp = tempdir().unwrap();
        let encryption_key: [u8; 32] = rand::random();

        let mut store = AuthCredentialStore::new(encryption_key, temp.path()).unwrap();

        store
            .set_profile(AuthProfile::new("a1", ProfileType::ApiKey, "anthropic"))
            .unwrap();
        store
            .set_profile(AuthProfile::new("o1", ProfileType::ApiKey, "openai"))
            .unwrap();
        store
            .set_profile(AuthProfile::new("a2", ProfileType::ApiKey, "anthropic"))
            .unwrap();

        let anthropic_profiles = store.profiles_for_target("anthropic");
        assert_eq!(anthropic_profiles.len(), 2);
    }
}
