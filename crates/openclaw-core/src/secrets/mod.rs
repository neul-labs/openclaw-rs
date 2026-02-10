//! Secrets management with encryption at rest.
//!
//! - `ApiKey`: Wrapper that prevents accidental logging
//! - `CredentialStore`: Encrypted storage for credentials
//! - `scrub_secrets`: Redact secrets from error messages

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use secrecy::{ExposeSecret, SecretBox};
use std::path::PathBuf;
use thiserror::Error;
use zeroize::Zeroize;

/// Errors from credential operations.
#[derive(Error, Debug)]
pub enum CredentialError {
    /// IO error reading/writing credentials.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Encryption/decryption failed.
    #[error("Crypto error: {0}")]
    Crypto(String),

    /// Invalid UTF-8 in decrypted data.
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// Credential not found.
    #[error("Credential not found: {0}")]
    NotFound(String),
}

/// API key wrapper that prevents accidental logging.
///
/// The inner value is wrapped with `secrecy::SecretBox` to ensure
/// it's not accidentally printed in logs or debug output.
#[derive(Clone)]
pub struct ApiKey(SecretBox<str>);

impl ApiKey {
    /// Create a new API key.
    #[must_use]
    pub fn new(key: String) -> Self {
        Self(SecretBox::new(key.into_boxed_str()))
    }

    /// Expose the secret for actual API calls.
    ///
    /// Use sparingly - only when actually sending to an API.
    #[must_use]
    pub fn expose(&self) -> &str {
        self.0.expose_secret()
    }
}

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApiKey([REDACTED])")
    }
}

impl std::fmt::Display for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

/// Credential storage with encryption at rest.
///
/// Uses AES-256-GCM for authenticated encryption.
pub struct CredentialStore {
    encryption_key: SecretBox<[u8; 32]>,
    store_path: PathBuf,
}

impl CredentialStore {
    /// Create a new credential store.
    ///
    /// # Arguments
    ///
    /// * `encryption_key` - 32-byte encryption key (derive from master password with Argon2)
    /// * `store_path` - Directory to store encrypted credentials
    #[must_use]
    pub fn new(encryption_key: [u8; 32], store_path: PathBuf) -> Self {
        Self {
            encryption_key: SecretBox::new(Box::new(encryption_key)),
            store_path,
        }
    }

    /// Store an encrypted credential.
    ///
    /// The credential is encrypted with AES-256-GCM and written to disk
    /// with restrictive permissions (0600 on Unix).
    ///
    /// # Errors
    ///
    /// Returns error if encryption or file write fails.
    pub fn store(&self, name: &str, credential: &ApiKey) -> Result<(), CredentialError> {
        // Ensure store directory exists
        std::fs::create_dir_all(&self.store_path)?;

        let encrypted = self.encrypt(credential.expose().as_bytes())?;

        let path = self.store_path.join(format!("{name}.enc"));
        std::fs::write(&path, &encrypted)?;

        // Set restrictive permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    /// Load and decrypt a credential.
    ///
    /// # Errors
    ///
    /// Returns error if file not found, decryption fails, or invalid UTF-8.
    pub fn load(&self, name: &str) -> Result<ApiKey, CredentialError> {
        let path = self.store_path.join(format!("{name}.enc"));

        if !path.exists() {
            return Err(CredentialError::NotFound(name.to_string()));
        }

        let encrypted = std::fs::read(&path)?;
        let mut decrypted = self.decrypt(&encrypted)?;

        let key = ApiKey::new(String::from_utf8(decrypted.clone())?);

        // Clear decrypted data from memory
        decrypted.zeroize();

        Ok(key)
    }

    /// Delete a stored credential.
    ///
    /// # Errors
    ///
    /// Returns error if file deletion fails.
    pub fn delete(&self, name: &str) -> Result<(), CredentialError> {
        let path = self.store_path.join(format!("{name}.enc"));
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// List all stored credential names.
    ///
    /// # Errors
    ///
    /// Returns error if directory read fails.
    pub fn list(&self) -> Result<Vec<String>, CredentialError> {
        if !self.store_path.exists() {
            return Ok(vec![]);
        }

        let mut names = Vec::new();
        for entry in std::fs::read_dir(&self.store_path)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                if let Some(name) = name.strip_suffix(".enc") {
                    names.push(name.to_string());
                }
            }
        }
        Ok(names)
    }

    /// Encrypt data with AES-256-GCM.
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, CredentialError> {
        let cipher = Aes256Gcm::new(self.encryption_key.expose_secret().into());

        // Generate random 12-byte nonce
        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| CredentialError::Crypto(e.to_string()))?;

        // Prepend nonce to ciphertext
        Ok([nonce_bytes.as_slice(), &ciphertext].concat())
    }

    /// Decrypt data with AES-256-GCM.
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CredentialError> {
        if data.len() < 12 {
            return Err(CredentialError::Crypto("Data too short".to_string()));
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let cipher = Aes256Gcm::new(self.encryption_key.expose_secret().into());
        let nonce = Nonce::from_slice(nonce_bytes);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CredentialError::Crypto(e.to_string()))
    }
}

/// Scrub secrets from error messages and logs.
///
/// Replaces values after known secret patterns with `[REDACTED]`.
///
/// # Arguments
///
/// * `text` - Text to scrub
/// * `patterns` - Patterns to look for (e.g., `["api_key=", "token="]`)
#[must_use]
pub fn scrub_secrets(text: &str, patterns: &[&str]) -> String {
    let mut result = text.to_string();

    for pattern in patterns {
        // Find all occurrences of the pattern
        let mut search_start = 0;
        while let Some(start) = result[search_start..].find(pattern) {
            let abs_start = search_start + start + pattern.len();

            // Find the end of the value (whitespace, quote, or end of string)
            let end = result[abs_start..]
                .find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == '&' || c == ',')
                .map_or(result.len(), |e| abs_start + e);

            // Replace the value with [REDACTED]
            result.replace_range(abs_start..end, "[REDACTED]");

            search_start = abs_start + "[REDACTED]".len();
        }
    }

    result
}

/// Common secret patterns to scrub from logs.
pub const COMMON_SECRET_PATTERNS: &[&str] = &[
    "api_key=",
    "apikey=",
    "api-key=",
    "token=",
    "secret=",
    "password=",
    "Authorization: Bearer ",
    "Authorization: Basic ",
    "x-api-key: ",
];

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_api_key_redaction() {
        let key = ApiKey::new("sk-secret-key-12345".to_string());

        // Debug output should be redacted
        assert_eq!(format!("{key:?}"), "ApiKey([REDACTED])");
        assert_eq!(format!("{key}"), "[REDACTED]");

        // But we can still expose when needed
        assert_eq!(key.expose(), "sk-secret-key-12345");
    }

    #[test]
    fn test_credential_store_roundtrip() {
        let temp = tempdir().unwrap();
        let encryption_key: [u8; 32] = rand::random();
        let store = CredentialStore::new(encryption_key, temp.path().to_path_buf());

        let original = ApiKey::new("my-secret-api-key".to_string());
        store.store("test-cred", &original).unwrap();

        let loaded = store.load("test-cred").unwrap();
        assert_eq!(loaded.expose(), "my-secret-api-key");
    }

    #[test]
    fn test_credential_not_found() {
        let temp = tempdir().unwrap();
        let encryption_key: [u8; 32] = rand::random();
        let store = CredentialStore::new(encryption_key, temp.path().to_path_buf());

        let result = store.load("nonexistent");
        assert!(matches!(result, Err(CredentialError::NotFound(_))));
    }

    #[test]
    fn test_credential_list() {
        let temp = tempdir().unwrap();
        let encryption_key: [u8; 32] = rand::random();
        let store = CredentialStore::new(encryption_key, temp.path().to_path_buf());

        store
            .store("cred1", &ApiKey::new("value1".to_string()))
            .unwrap();
        store
            .store("cred2", &ApiKey::new("value2".to_string()))
            .unwrap();

        let names = store.list().unwrap();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"cred1".to_string()));
        assert!(names.contains(&"cred2".to_string()));
    }

    #[test]
    fn test_scrub_secrets() {
        let text = "Error: api_key=sk-12345 failed with token=abc123";
        let scrubbed = scrub_secrets(text, &["api_key=", "token="]);
        assert_eq!(
            scrubbed,
            "Error: api_key=[REDACTED] failed with token=[REDACTED]"
        );
    }

    #[test]
    fn test_scrub_secrets_with_quotes() {
        let text = r#"{"api_key":"sk-secret","other":"value"}"#;
        let scrubbed = scrub_secrets(text, &["api_key\":\""]);
        assert!(scrubbed.contains("[REDACTED]"));
        assert!(!scrubbed.contains("sk-secret"));
    }
}
