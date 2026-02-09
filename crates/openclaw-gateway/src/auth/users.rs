//! User model and storage.

use std::path::Path;
use std::sync::Arc;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::AuthError;

/// User role for access control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    /// Full administrative access.
    Admin,
    /// Can manage sessions and view all data.
    Operator,
    /// Read-only access.
    Viewer,
}

impl UserRole {
    /// Check if this role has admin privileges.
    #[must_use]
    pub fn is_admin(&self) -> bool {
        matches!(self, Self::Admin)
    }

    /// Check if this role can manage sessions.
    #[must_use]
    pub fn can_manage_sessions(&self) -> bool {
        matches!(self, Self::Admin | Self::Operator)
    }

    /// Check if this role can view data.
    #[must_use]
    pub fn can_view(&self) -> bool {
        true // All roles can view
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admin => write!(f, "admin"),
            Self::Operator => write!(f, "operator"),
            Self::Viewer => write!(f, "viewer"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = AuthError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(Self::Admin),
            "operator" => Ok(Self::Operator),
            "viewer" => Ok(Self::Viewer),
            _ => Err(AuthError::Config(format!("Unknown role: {s}"))),
        }
    }
}

/// User account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user ID.
    pub id: String,
    /// Username for login.
    pub username: String,
    /// Optional email address.
    pub email: Option<String>,
    /// Argon2 password hash (stored in DB, not exposed in public API).
    pub password_hash: String,
    /// User role.
    pub role: UserRole,
    /// When the user was created.
    pub created_at: DateTime<Utc>,
    /// When the user last logged in.
    pub last_login: Option<DateTime<Utc>>,
    /// Whether the account is active.
    pub active: bool,
}

impl User {
    /// Create a new user with the given credentials.
    ///
    /// # Errors
    ///
    /// Returns error if password hashing fails.
    pub fn new(
        username: impl Into<String>,
        password: &str,
        role: UserRole,
    ) -> Result<Self, AuthError> {
        let username = username.into();
        let id = format!("user_{}", uuid_v4());
        let password_hash = hash_password(password)?;

        Ok(Self {
            id,
            username,
            email: None,
            password_hash,
            role,
            created_at: Utc::now(),
            last_login: None,
            active: true,
        })
    }

    /// Verify a password against this user's hash.
    ///
    /// # Errors
    ///
    /// Returns error if password doesn't match.
    pub fn verify_password(&self, password: &str) -> Result<(), AuthError> {
        verify_password(password, &self.password_hash)
    }

    /// Update the user's password.
    ///
    /// # Errors
    ///
    /// Returns error if password hashing fails.
    pub fn set_password(&mut self, password: &str) -> Result<(), AuthError> {
        self.password_hash = hash_password(password)?;
        Ok(())
    }

    /// Create a safe version of user for API responses (no password hash).
    #[must_use]
    pub fn to_public(&self) -> PublicUser {
        PublicUser {
            id: self.id.clone(),
            username: self.username.clone(),
            email: self.email.clone(),
            role: self.role,
            created_at: self.created_at,
            last_login: self.last_login,
            active: self.active,
        }
    }
}

/// Public user representation (for API responses).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicUser {
    /// Unique user ID.
    pub id: String,
    /// Username.
    pub username: String,
    /// Email address.
    pub email: Option<String>,
    /// User role.
    pub role: UserRole,
    /// When created.
    pub created_at: DateTime<Utc>,
    /// Last login time.
    pub last_login: Option<DateTime<Utc>>,
    /// Whether active.
    pub active: bool,
}

/// User store backed by sled.
pub struct UserStore {
    db: sled::Db,
    tree: sled::Tree,
}

impl UserStore {
    /// Open or create a user store at the given path.
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be opened.
    pub fn open(path: &Path) -> Result<Self, AuthError> {
        let db = sled::open(path.join("auth"))
            .map_err(|e| AuthError::Storage(format!("Failed to open auth database: {e}")))?;

        let tree = db
            .open_tree("users")
            .map_err(|e| AuthError::Storage(format!("Failed to open users tree: {e}")))?;

        Ok(Self { db, tree })
    }

    /// Create a new user store with an existing sled database.
    ///
    /// # Errors
    ///
    /// Returns error if tree cannot be opened.
    pub fn with_db(db: sled::Db) -> Result<Self, AuthError> {
        let tree = db
            .open_tree("users")
            .map_err(|e| AuthError::Storage(format!("Failed to open users tree: {e}")))?;

        Ok(Self { db, tree })
    }

    /// Get the underlying sled database.
    #[must_use]
    pub fn db(&self) -> &sled::Db {
        &self.db
    }

    /// Check if any users exist.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    /// Count total users.
    #[must_use]
    pub fn count(&self) -> usize {
        // Count entries that don't start with "idx:" prefix
        self.tree
            .iter()
            .filter(|r| {
                r.as_ref()
                    .map(|(k, _)| !k.starts_with(b"idx:"))
                    .unwrap_or(false)
            })
            .count()
    }

    /// Create a new user.
    ///
    /// # Errors
    ///
    /// Returns error if user already exists or storage fails.
    pub fn create(&self, user: &User) -> Result<(), AuthError> {
        // Check if username already exists
        if self.get_by_username(&user.username)?.is_some() {
            return Err(AuthError::UserExists(user.username.clone()));
        }

        let key = user.id.as_bytes();
        let value = serde_json::to_vec(user)
            .map_err(|e| AuthError::Storage(format!("Serialization error: {e}")))?;

        self.tree
            .insert(key, value)
            .map_err(|e| AuthError::Storage(format!("Insert error: {e}")))?;

        // Create username -> id index
        let index_key = format!("idx:username:{}", user.username);
        self.tree
            .insert(index_key.as_bytes(), user.id.as_bytes())
            .map_err(|e| AuthError::Storage(format!("Index error: {e}")))?;

        self.tree
            .flush()
            .map_err(|e| AuthError::Storage(format!("Flush error: {e}")))?;

        Ok(())
    }

    /// Get a user by ID.
    ///
    /// # Errors
    ///
    /// Returns error if storage fails.
    pub fn get(&self, id: &str) -> Result<Option<User>, AuthError> {
        let key = id.as_bytes();
        match self.tree.get(key) {
            Ok(Some(value)) => {
                let user: User = serde_json::from_slice(&value)
                    .map_err(|e| AuthError::Storage(format!("Deserialization error: {e}")))?;
                Ok(Some(user))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(AuthError::Storage(format!("Get error: {e}"))),
        }
    }

    /// Get a user by username.
    ///
    /// # Errors
    ///
    /// Returns error if storage fails.
    pub fn get_by_username(&self, username: &str) -> Result<Option<User>, AuthError> {
        let index_key = format!("idx:username:{username}");
        match self.tree.get(index_key.as_bytes()) {
            Ok(Some(id_bytes)) => {
                let id = String::from_utf8_lossy(&id_bytes);
                self.get(&id)
            }
            Ok(None) => Ok(None),
            Err(e) => Err(AuthError::Storage(format!("Index lookup error: {e}"))),
        }
    }

    /// Update an existing user.
    ///
    /// # Errors
    ///
    /// Returns error if user doesn't exist or storage fails.
    pub fn update(&self, user: &User) -> Result<(), AuthError> {
        // Verify user exists
        if self.get(&user.id)?.is_none() {
            return Err(AuthError::UserNotFound(user.id.clone()));
        }

        let key = user.id.as_bytes();
        let value = serde_json::to_vec(user)
            .map_err(|e| AuthError::Storage(format!("Serialization error: {e}")))?;

        self.tree
            .insert(key, value)
            .map_err(|e| AuthError::Storage(format!("Update error: {e}")))?;

        self.tree
            .flush()
            .map_err(|e| AuthError::Storage(format!("Flush error: {e}")))?;

        Ok(())
    }

    /// Delete a user.
    ///
    /// # Errors
    ///
    /// Returns error if storage fails.
    pub fn delete(&self, id: &str) -> Result<bool, AuthError> {
        // Get user first to remove index
        if let Some(user) = self.get(id)? {
            let index_key = format!("idx:username:{}", user.username);
            self.tree
                .remove(index_key.as_bytes())
                .map_err(|e| AuthError::Storage(format!("Index remove error: {e}")))?;
        }

        let removed = self
            .tree
            .remove(id.as_bytes())
            .map_err(|e| AuthError::Storage(format!("Delete error: {e}")))?
            .is_some();

        self.tree
            .flush()
            .map_err(|e| AuthError::Storage(format!("Flush error: {e}")))?;

        Ok(removed)
    }

    /// List all users.
    ///
    /// # Errors
    ///
    /// Returns error if storage fails.
    pub fn list(&self) -> Result<Vec<User>, AuthError> {
        let mut users = Vec::new();

        for result in self.tree.iter() {
            let (key, value) = result.map_err(|e| AuthError::Storage(format!("Iter error: {e}")))?;

            // Skip index entries
            if key.starts_with(b"idx:") {
                continue;
            }

            let user: User = serde_json::from_slice(&value)
                .map_err(|e| AuthError::Storage(format!("Deserialization error: {e}")))?;
            users.push(user);
        }

        Ok(users)
    }

    /// Update last login time for a user.
    ///
    /// # Errors
    ///
    /// Returns error if user doesn't exist or storage fails.
    pub fn update_last_login(&self, id: &str) -> Result<(), AuthError> {
        let mut user = self
            .get(id)?
            .ok_or_else(|| AuthError::UserNotFound(id.to_string()))?;

        user.last_login = Some(Utc::now());
        self.update(&user)
    }
}

/// Hash a password using Argon2id.
fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AuthError::Config(format!("Password hashing failed: {e}")))
}

/// Verify a password against a hash.
fn verify_password(password: &str, hash: &str) -> Result<(), AuthError> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| AuthError::Config(format!("Invalid hash: {e}")))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AuthError::InvalidCredentials)
}

/// Generate a simple UUID v4.
fn uuid_v4() -> String {
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 16];
    rng.fill_bytes(&mut bytes);

    // Set version (4) and variant bits
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_user_creation() {
        let user = User::new("testuser", "password123", UserRole::Admin).unwrap();
        assert_eq!(user.username, "testuser");
        assert!(user.id.starts_with("user_"));
        assert!(user.active);
        assert_eq!(user.role, UserRole::Admin);
    }

    #[test]
    fn test_password_verification() {
        let user = User::new("testuser", "password123", UserRole::Admin).unwrap();
        assert!(user.verify_password("password123").is_ok());
        assert!(user.verify_password("wrongpassword").is_err());
    }

    #[test]
    fn test_user_store() {
        let temp_dir = TempDir::new().unwrap();
        let store = UserStore::open(temp_dir.path()).unwrap();

        assert!(store.is_empty());

        let user = User::new("admin", "secret", UserRole::Admin).unwrap();
        store.create(&user).unwrap();

        assert!(!store.is_empty());
        assert_eq!(store.count(), 1);

        let loaded = store.get(&user.id).unwrap().unwrap();
        assert_eq!(loaded.username, "admin");

        let by_name = store.get_by_username("admin").unwrap().unwrap();
        assert_eq!(by_name.id, user.id);
    }

    #[test]
    fn test_user_roles() {
        assert!(UserRole::Admin.is_admin());
        assert!(!UserRole::Operator.is_admin());
        assert!(!UserRole::Viewer.is_admin());

        assert!(UserRole::Admin.can_manage_sessions());
        assert!(UserRole::Operator.can_manage_sessions());
        assert!(!UserRole::Viewer.can_manage_sessions());
    }

    #[test]
    fn test_duplicate_user() {
        let temp_dir = TempDir::new().unwrap();
        let store = UserStore::open(temp_dir.path()).unwrap();

        let user1 = User::new("admin", "secret1", UserRole::Admin).unwrap();
        store.create(&user1).unwrap();

        let user2 = User::new("admin", "secret2", UserRole::Operator).unwrap();
        let result = store.create(&user2);

        assert!(matches!(result, Err(AuthError::UserExists(_))));
    }
}
