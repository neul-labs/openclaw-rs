//! Encrypted credential storage bindings.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use openclaw_core::secrets::CredentialStore as RustCredentialStore;

use super::api_key::NodeApiKey;
use crate::error::OpenClawError;

/// Encrypted credential storage.
///
/// Stores API keys and other credentials encrypted with AES-256-GCM.
/// Credentials are stored on disk with restrictive permissions.
///
/// ```javascript
/// // Create with 32-byte hex encryption key
/// const store = new CredentialStore(
///   '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef',
///   '/path/to/credentials'
/// );
///
/// // Store a credential
/// const key = new NodeApiKey('sk-secret');
/// await store.store('anthropic', key);
///
/// // Load it back
/// const loaded = await store.load('anthropic');
/// ```
#[napi]
pub struct CredentialStore {
    inner: Arc<RwLock<RustCredentialStore>>,
}

#[napi]
impl CredentialStore {
    /// Create a new credential store.
    ///
    /// # Arguments
    ///
    /// * `encryption_key_hex` - 32-byte encryption key as hex string (64 characters)
    /// * `store_path` - Directory to store encrypted credentials
    ///
    /// # Example
    ///
    /// ```javascript
    /// // Generate a key: require('crypto').randomBytes(32).toString('hex')
    /// const store = new CredentialStore(keyHex, '/path/to/creds');
    /// ```
    #[napi(constructor)]
    pub fn new(encryption_key_hex: String, store_path: String) -> Result<Self> {
        let key_bytes = hex::decode(&encryption_key_hex)
            .map_err(|e| OpenClawError::new("INVALID_KEY", format!("Invalid hex key: {e}")))?;

        if key_bytes.len() != 32 {
            return Err(OpenClawError::new(
                "INVALID_KEY",
                format!(
                    "Encryption key must be 32 bytes (64 hex chars), got {}",
                    key_bytes.len()
                ),
            )
            .into());
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);

        let store = RustCredentialStore::new(key, PathBuf::from(store_path));

        Ok(Self {
            inner: Arc::new(RwLock::new(store)),
        })
    }

    /// Store an encrypted credential.
    ///
    /// The credential is encrypted with AES-256-GCM and written to disk
    /// with restrictive permissions (0600 on Unix).
    #[napi]
    pub async fn store(&self, name: String, api_key: &NodeApiKey) -> Result<()> {
        let store = self.inner.read().await;
        store
            .store(&name, &api_key.inner)
            .map_err(|e| OpenClawError::from_credential_error(e).into())
    }

    /// Load and decrypt a credential.
    ///
    /// Returns the decrypted API key wrapped in NodeApiKey for safety.
    #[napi]
    pub async fn load(&self, name: String) -> Result<NodeApiKey> {
        let store = self.inner.read().await;
        let key = store
            .load(&name)
            .map_err(|e| OpenClawError::from_credential_error(e))?;
        Ok(NodeApiKey { inner: key })
    }

    /// Delete a stored credential.
    #[napi]
    pub async fn delete(&self, name: String) -> Result<()> {
        let store = self.inner.read().await;
        store
            .delete(&name)
            .map_err(|e| OpenClawError::from_credential_error(e).into())
    }

    /// List all stored credential names.
    #[napi]
    pub async fn list(&self) -> Result<Vec<String>> {
        let store = self.inner.read().await;
        store
            .list()
            .map_err(|e| OpenClawError::from_credential_error(e).into())
    }

    /// Check if a credential exists.
    #[napi]
    pub async fn exists(&self, name: String) -> Result<bool> {
        let names = self.list().await?;
        Ok(names.contains(&name))
    }
}
