//! First-run setup and bootstrap management.

use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::AuthError;
use super::users::{User, UserRole, UserStore};

/// Bootstrap token validity duration.
const BOOTSTRAP_TOKEN_VALIDITY: Duration = Duration::from_secs(3600); // 1 hour

/// Setup status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupStatus {
    /// Whether the system is initialized (has at least one admin).
    pub initialized: bool,
    /// Number of users configured.
    pub user_count: usize,
    /// Whether a bootstrap token is active.
    pub bootstrap_active: bool,
    /// Bootstrap URL (only if bootstrap is active and not initialized).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup_url: Option<String>,
}

/// Manages the first-run bootstrap process.
pub struct BootstrapManager {
    /// Bootstrap token (if active).
    token: Option<BootstrapToken>,
    /// When the manager was created.
    created_at: Instant,
}

/// Internal bootstrap token state.
struct BootstrapToken {
    /// The token value (URL-safe base64).
    value: String,
    /// When the token was created.
    created_at: Instant,
    /// When the token expires.
    expires_at: Instant,
}

impl BootstrapManager {
    /// Create a new bootstrap manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            token: None,
            created_at: Instant::now(),
        }
    }

    /// Check if setup is required and generate bootstrap token if needed.
    ///
    /// Returns the bootstrap token if one was generated.
    pub fn check_and_generate(&mut self, user_store: &UserStore) -> Option<String> {
        if !user_store.is_empty() {
            // System already initialized
            self.token = None;
            return None;
        }

        // Check if we already have a valid token
        if let Some(ref token) = self.token {
            if Instant::now() < token.expires_at {
                return Some(token.value.clone());
            }
        }

        // Generate new bootstrap token
        let token_value = Self::generate_token();
        let now = Instant::now();

        self.token = Some(BootstrapToken {
            value: token_value.clone(),
            created_at: now,
            expires_at: now + BOOTSTRAP_TOKEN_VALIDITY,
        });

        Some(token_value)
    }

    /// Validate a bootstrap token.
    pub fn validate_token(&self, token: &str) -> Result<(), AuthError> {
        match &self.token {
            Some(bt) if bt.value == token && Instant::now() < bt.expires_at => Ok(()),
            Some(_) => Err(AuthError::InvalidBootstrapToken),
            None => Err(AuthError::InvalidBootstrapToken),
        }
    }

    /// Complete setup with the bootstrap token.
    ///
    /// Creates the initial admin user and invalidates the bootstrap token.
    ///
    /// # Errors
    ///
    /// Returns error if token is invalid or user creation fails.
    pub fn complete_setup(
        &mut self,
        user_store: &UserStore,
        token: &str,
        username: &str,
        password: &str,
        email: Option<String>,
    ) -> Result<User, AuthError> {
        // Validate token
        self.validate_token(token)?;

        // Check system isn't already initialized
        if !user_store.is_empty() {
            return Err(AuthError::Config("System already initialized".to_string()));
        }

        // Create admin user
        let mut admin = User::new(username, password, UserRole::Admin)?;
        admin.email = email;

        user_store.create(&admin)?;

        // Invalidate bootstrap token
        self.token = None;

        tracing::info!(
            username = %admin.username,
            "Initial admin user created via bootstrap"
        );

        Ok(admin)
    }

    /// Get setup status.
    pub fn status(&self, user_store: &UserStore, base_url: Option<&str>) -> SetupStatus {
        let initialized = !user_store.is_empty();
        let bootstrap_active = self
            .token
            .as_ref()
            .is_some_and(|t| Instant::now() < t.expires_at);

        let setup_url = if !initialized && bootstrap_active {
            self.token
                .as_ref()
                .and_then(|t| base_url.map(|url| format!("{url}/setup?token={}", t.value)))
        } else {
            None
        };

        SetupStatus {
            initialized,
            user_count: user_store.count(),
            bootstrap_active,
            setup_url,
        }
    }

    /// Check if bootstrap is required (no users exist).
    #[must_use]
    pub fn is_required(user_store: &UserStore) -> bool {
        user_store.is_empty()
    }

    /// Get time remaining on bootstrap token (if any).
    #[must_use]
    pub fn token_time_remaining(&self) -> Option<Duration> {
        self.token.as_ref().and_then(|t| {
            let now = Instant::now();
            if now < t.expires_at {
                Some(t.expires_at - now)
            } else {
                None
            }
        })
    }

    /// Invalidate the current bootstrap token.
    pub fn invalidate_token(&mut self) {
        self.token = None;
    }

    /// Generate a secure random token.
    fn generate_token() -> String {
        use rand::RngCore;
        let mut bytes = [0u8; 48];
        rand::thread_rng().fill_bytes(&mut bytes);
        // Use URL-safe base64 encoding
        base64_url_encode(&bytes)
    }

    /// Print bootstrap information to console.
    pub fn print_bootstrap_info(&self, base_url: &str) {
        if let Some(ref token) = self.token {
            let remaining = self.token_time_remaining().unwrap_or_default();
            let minutes = remaining.as_secs() / 60;

            println!();
            println!("┌─────────────────────────────────────────────────────────┐");
            println!("│  OpenClaw Gateway Started                                │");
            println!("│                                                          │");
            println!("│  No admin user configured. Complete setup at:            │");
            println!("│  {}/setup?token={}...", base_url, &token.value[..16]);
            println!("│                                                          │");
            println!("│  Or via CLI:                                             │");
            println!("│  openclaw admin create --username admin --password ...   │");
            println!("│                                                          │");
            println!(
                "│  Bootstrap token expires in {} minutes.                  │",
                minutes
            );
            println!("└─────────────────────────────────────────────────────────┘");
            println!();
        }
    }
}

impl Default for BootstrapManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Auto-setup from environment variables.
///
/// Checks for `OPENCLAW_ADMIN_USERNAME` and `OPENCLAW_ADMIN_PASSWORD` env vars
/// and creates admin user if both are set and no users exist.
///
/// # Errors
///
/// Returns error if user creation fails.
pub fn auto_setup_from_env(user_store: &UserStore) -> Result<Option<User>, AuthError> {
    // Only auto-setup if no users exist
    if !user_store.is_empty() {
        return Ok(None);
    }

    let username = match std::env::var("OPENCLAW_ADMIN_USERNAME") {
        Ok(u) if !u.is_empty() => u,
        _ => return Ok(None),
    };

    let password = match std::env::var("OPENCLAW_ADMIN_PASSWORD") {
        Ok(p) if !p.is_empty() => p,
        _ => return Ok(None),
    };

    let admin = User::new(&username, &password, UserRole::Admin)?;
    user_store.create(&admin)?;

    tracing::info!(
        username = %admin.username,
        "Admin user created from environment variables"
    );

    Ok(Some(admin))
}

/// Generate a secure random password.
#[must_use]
pub fn generate_password(length: usize) -> String {
    use rand::RngCore;
    const CHARSET: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| {
            let idx = (rng.next_u32() as usize) % CHARSET.len();
            CHARSET[idx] as char
        })
        .collect()
}

/// URL-safe base64 encoding without padding.
fn base64_url_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    let mut result = String::with_capacity((data.len() * 4 + 2) / 3);

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        }
        if chunk.len() > 2 {
            result.push(ALPHABET[b2 & 0x3f] as char);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_bootstrap_manager_new() {
        let manager = BootstrapManager::new();
        assert!(manager.token.is_none());
    }

    #[test]
    fn test_generate_token() {
        let token1 = BootstrapManager::generate_token();
        let token2 = BootstrapManager::generate_token();

        assert!(!token1.is_empty());
        assert_ne!(token1, token2);
        // URL-safe base64 should not contain + or /
        assert!(!token1.contains('+'));
        assert!(!token1.contains('/'));
    }

    #[test]
    fn test_check_and_generate() {
        let temp_dir = TempDir::new().unwrap();
        let store = UserStore::open(temp_dir.path()).unwrap();
        let mut manager = BootstrapManager::new();

        // Should generate token when no users
        let token = manager.check_and_generate(&store);
        assert!(token.is_some());

        // Should return same token on second call
        let token2 = manager.check_and_generate(&store);
        assert_eq!(token, token2);
    }

    #[test]
    fn test_no_bootstrap_when_users_exist() {
        let temp_dir = TempDir::new().unwrap();
        let store = UserStore::open(temp_dir.path()).unwrap();

        // Create a user
        let user = User::new("admin", "password", UserRole::Admin).unwrap();
        store.create(&user).unwrap();

        let mut manager = BootstrapManager::new();
        let token = manager.check_and_generate(&store);
        assert!(token.is_none());
    }

    #[test]
    fn test_complete_setup() {
        let temp_dir = TempDir::new().unwrap();
        let store = UserStore::open(temp_dir.path()).unwrap();
        let mut manager = BootstrapManager::new();

        let token = manager.check_and_generate(&store).unwrap();

        let admin = manager
            .complete_setup(&store, &token, "admin", "secret123", None)
            .unwrap();

        assert_eq!(admin.username, "admin");
        assert_eq!(admin.role, UserRole::Admin);
        assert!(!store.is_empty());

        // Token should be invalidated
        assert!(manager.token.is_none());
    }

    #[test]
    fn test_invalid_bootstrap_token() {
        let temp_dir = TempDir::new().unwrap();
        let store = UserStore::open(temp_dir.path()).unwrap();
        let mut manager = BootstrapManager::new();

        let _token = manager.check_and_generate(&store).unwrap();

        let result = manager.complete_setup(&store, "wrong_token", "admin", "secret", None);
        assert!(matches!(result, Err(AuthError::InvalidBootstrapToken)));
    }

    #[test]
    fn test_setup_status() {
        let temp_dir = TempDir::new().unwrap();
        let store = UserStore::open(temp_dir.path()).unwrap();
        let mut manager = BootstrapManager::new();

        // Before bootstrap
        let status = manager.status(&store, Some("http://localhost:18789"));
        assert!(!status.initialized);
        assert!(!status.bootstrap_active);

        // Generate token
        manager.check_and_generate(&store);
        let status = manager.status(&store, Some("http://localhost:18789"));
        assert!(!status.initialized);
        assert!(status.bootstrap_active);
        assert!(status.setup_url.is_some());
    }

    #[test]
    fn test_generate_password() {
        let pwd1 = generate_password(16);
        let pwd2 = generate_password(16);

        assert_eq!(pwd1.len(), 16);
        assert_ne!(pwd1, pwd2);
    }
}
