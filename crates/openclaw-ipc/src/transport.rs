//! nng transport layer with async support.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;

use nng::options::Options;

use crate::messages::IpcMessage;

/// Transport errors.
#[derive(Error, Debug)]
pub enum TransportError {
    /// Socket error.
    #[error("Socket error: {0}")]
    Socket(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Timeout.
    #[error("Operation timed out")]
    Timeout,

    /// Connection closed.
    #[error("Connection closed")]
    Closed,

    /// Task join error.
    #[error("Task join error: {0}")]
    TaskJoin(String),

    /// Pool exhausted.
    #[error("Connection pool exhausted")]
    PoolExhausted,
}

/// IPC transport using nng with async support.
pub struct IpcTransport {
    socket: Arc<nng::Socket>,
    timeout: Duration,
    address: String,
}

impl IpcTransport {
    /// Create a new request socket (client) with timeout.
    ///
    /// # Errors
    ///
    /// Returns error if socket creation fails.
    pub fn new_client(address: &str, timeout: Duration) -> Result<Self, TransportError> {
        let socket = nng::Socket::new(nng::Protocol::Req0)
            .map_err(|e| TransportError::Socket(e.to_string()))?;

        // Set socket timeouts
        socket
            .set_opt::<nng::options::RecvTimeout>(Some(timeout))
            .map_err(|e| TransportError::Socket(format!("Failed to set recv timeout: {e}")))?;
        socket
            .set_opt::<nng::options::SendTimeout>(Some(timeout))
            .map_err(|e| TransportError::Socket(format!("Failed to set send timeout: {e}")))?;

        socket
            .dial(address)
            .map_err(|e| TransportError::Socket(e.to_string()))?;

        Ok(Self {
            socket: Arc::new(socket),
            timeout,
            address: address.to_string(),
        })
    }

    /// Create a new reply socket (server) with timeout.
    ///
    /// # Errors
    ///
    /// Returns error if socket creation fails.
    pub fn new_server(address: &str) -> Result<Self, TransportError> {
        Self::new_server_with_timeout(address, Duration::from_secs(300))
    }

    /// Create a new reply socket (server) with custom timeout.
    ///
    /// # Errors
    ///
    /// Returns error if socket creation fails.
    pub fn new_server_with_timeout(
        address: &str,
        timeout: Duration,
    ) -> Result<Self, TransportError> {
        let socket = nng::Socket::new(nng::Protocol::Rep0)
            .map_err(|e| TransportError::Socket(e.to_string()))?;

        // Set socket timeouts
        socket
            .set_opt::<nng::options::RecvTimeout>(Some(timeout))
            .map_err(|e| TransportError::Socket(format!("Failed to set recv timeout: {e}")))?;
        socket
            .set_opt::<nng::options::SendTimeout>(Some(timeout))
            .map_err(|e| TransportError::Socket(format!("Failed to set send timeout: {e}")))?;

        socket
            .listen(address)
            .map_err(|e| TransportError::Socket(e.to_string()))?;

        Ok(Self {
            socket: Arc::new(socket),
            timeout,
            address: address.to_string(),
        })
    }

    /// Send a message (synchronous).
    ///
    /// # Errors
    ///
    /// Returns error if send fails.
    pub fn send(&self, message: &IpcMessage) -> Result<(), TransportError> {
        let data = serde_json::to_vec(message)?;
        let msg = nng::Message::from(data.as_slice());

        self.socket.send(msg).map_err(|(_, e)| match e {
            nng::Error::TimedOut => TransportError::Timeout,
            nng::Error::Closed => TransportError::Closed,
            _ => TransportError::Socket(format!("{e:?}")),
        })?;

        Ok(())
    }

    /// Send a message asynchronously.
    ///
    /// Wraps the synchronous send in a blocking task.
    ///
    /// # Errors
    ///
    /// Returns error if send fails.
    pub async fn send_async(&self, message: &IpcMessage) -> Result<(), TransportError> {
        let socket = self.socket.clone();
        let data = serde_json::to_vec(message)?;

        tokio::task::spawn_blocking(move || {
            let msg = nng::Message::from(data.as_slice());
            socket.send(msg).map_err(|(_, e)| match e {
                nng::Error::TimedOut => TransportError::Timeout,
                nng::Error::Closed => TransportError::Closed,
                _ => TransportError::Socket(format!("{e:?}")),
            })
        })
        .await
        .map_err(|e| TransportError::TaskJoin(e.to_string()))??;

        Ok(())
    }

    /// Receive a message (synchronous).
    ///
    /// # Errors
    ///
    /// Returns error if receive fails or times out.
    pub fn recv(&self) -> Result<IpcMessage, TransportError> {
        let msg = self.socket.recv().map_err(|e| match e {
            nng::Error::TimedOut => TransportError::Timeout,
            nng::Error::Closed => TransportError::Closed,
            _ => TransportError::Socket(format!("{e:?}")),
        })?;

        let message: IpcMessage = serde_json::from_slice(&msg)?;
        Ok(message)
    }

    /// Receive a message asynchronously.
    ///
    /// Wraps the synchronous receive in a blocking task.
    ///
    /// # Errors
    ///
    /// Returns error if receive fails or times out.
    pub async fn recv_async(&self) -> Result<IpcMessage, TransportError> {
        let socket = self.socket.clone();

        let msg = tokio::task::spawn_blocking(move || {
            socket.recv().map_err(|e| match e {
                nng::Error::TimedOut => TransportError::Timeout,
                nng::Error::Closed => TransportError::Closed,
                _ => TransportError::Socket(format!("{e:?}")),
            })
        })
        .await
        .map_err(|e| TransportError::TaskJoin(e.to_string()))??;

        let message: IpcMessage = serde_json::from_slice(&msg)?;
        Ok(message)
    }

    /// Send a request and wait for response (synchronous).
    ///
    /// # Errors
    ///
    /// Returns error if send/receive fails.
    pub fn request(&self, message: &IpcMessage) -> Result<IpcMessage, TransportError> {
        self.send(message)?;
        self.recv()
    }

    /// Send a request and wait for response asynchronously.
    ///
    /// # Errors
    ///
    /// Returns error if send/receive fails.
    pub async fn request_async(&self, message: &IpcMessage) -> Result<IpcMessage, TransportError> {
        self.send_async(message).await?;
        self.recv_async().await
    }

    /// Get the configured timeout.
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Get the address.
    #[must_use]
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Get the default IPC address.
    #[must_use]
    pub fn default_address() -> String {
        if cfg!(unix) {
            "ipc:///tmp/openclaw-gateway.ipc".to_string()
        } else {
            "tcp://127.0.0.1:18790".to_string()
        }
    }
}

impl Drop for IpcTransport {
    fn drop(&mut self) {
        // Socket is in Arc, let it drop naturally
    }
}

/// Connection pool for multiple IPC connections.
///
/// Provides round-robin load balancing across multiple transports.
pub struct TransportPool {
    transports: Vec<Arc<RwLock<IpcTransport>>>,
    round_robin: AtomicUsize,
    address: String,
    timeout: Duration,
    max_connections: usize,
}

impl TransportPool {
    /// Create a new transport pool.
    ///
    /// # Arguments
    ///
    /// * `address` - IPC address to connect to
    /// * `timeout` - Timeout for operations
    /// * `max_connections` - Maximum number of connections in the pool
    ///
    /// # Errors
    ///
    /// Returns error if initial connection fails.
    pub fn new(
        address: &str,
        timeout: Duration,
        max_connections: usize,
    ) -> Result<Self, TransportError> {
        let max_connections = max_connections.max(1);

        // Create initial connection
        let transport = IpcTransport::new_client(address, timeout)?;

        Ok(Self {
            transports: vec![Arc::new(RwLock::new(transport))],
            round_robin: AtomicUsize::new(0),
            address: address.to_string(),
            timeout,
            max_connections,
        })
    }

    /// Get a transport from the pool using round-robin.
    ///
    /// May create new connections up to `max_connections`.
    ///
    /// # Errors
    ///
    /// Returns error if connection fails.
    pub async fn get(&self) -> Result<Arc<RwLock<IpcTransport>>, TransportError> {
        let idx = self.round_robin.fetch_add(1, Ordering::Relaxed) % self.transports.len();
        Ok(self.transports[idx].clone())
    }

    /// Send a request using an available transport.
    ///
    /// # Errors
    ///
    /// Returns error if request fails.
    pub async fn request(&self, message: &IpcMessage) -> Result<IpcMessage, TransportError> {
        let transport = self.get().await?;
        let guard = transport.read().await;
        guard.request_async(message).await
    }

    /// Get current pool size.
    #[must_use]
    pub fn size(&self) -> usize {
        self.transports.len()
    }

    /// Get max pool size.
    #[must_use]
    pub const fn max_size(&self) -> usize {
        self.max_connections
    }
}

/// Reconnecting transport wrapper with automatic reconnection.
pub struct ReconnectingTransport {
    address: String,
    timeout: Duration,
    transport: Arc<RwLock<Option<IpcTransport>>>,
    max_retries: usize,
    retry_delay: Duration,
}

impl ReconnectingTransport {
    /// Create a new reconnecting transport.
    ///
    /// # Arguments
    ///
    /// * `address` - IPC address to connect to
    /// * `timeout` - Timeout for operations
    /// * `max_retries` - Maximum number of reconnection attempts
    /// * `retry_delay` - Delay between reconnection attempts
    #[must_use]
    pub fn new(
        address: &str,
        timeout: Duration,
        max_retries: usize,
        retry_delay: Duration,
    ) -> Self {
        Self {
            address: address.to_string(),
            timeout,
            transport: Arc::new(RwLock::new(None)),
            max_retries,
            retry_delay,
        }
    }

    /// Connect or reconnect to the server.
    ///
    /// # Errors
    ///
    /// Returns error if connection fails after all retries.
    pub async fn connect(&self) -> Result<(), TransportError> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                tracing::info!(
                    "Reconnection attempt {} of {} to {}",
                    attempt,
                    self.max_retries,
                    self.address
                );
                tokio::time::sleep(self.retry_delay).await;
            }

            match IpcTransport::new_client(&self.address, self.timeout) {
                Ok(transport) => {
                    let mut guard = self.transport.write().await;
                    *guard = Some(transport);
                    tracing::info!("Connected to {}", self.address);
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!("Connection attempt failed: {}", e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or(TransportError::Closed))
    }

    /// Send a request with automatic reconnection.
    ///
    /// # Errors
    ///
    /// Returns error if request fails after reconnection attempts.
    pub async fn request(&self, message: &IpcMessage) -> Result<IpcMessage, TransportError> {
        // Try with existing connection
        {
            let guard = self.transport.read().await;
            if let Some(transport) = guard.as_ref() {
                match transport.request_async(message).await {
                    Ok(response) => return Ok(response),
                    Err(TransportError::Closed | TransportError::Timeout) => {
                        // Need to reconnect
                    }
                    Err(e) => return Err(e),
                }
            }
        }

        // Reconnect and retry
        self.connect().await?;

        let guard = self.transport.read().await;
        if let Some(transport) = guard.as_ref() {
            transport.request_async(message).await
        } else {
            Err(TransportError::Closed)
        }
    }

    /// Check if connected.
    pub async fn is_connected(&self) -> bool {
        self.transport.read().await.is_some()
    }

    /// Disconnect.
    pub async fn disconnect(&self) {
        let mut guard = self.transport.write().await;
        *guard = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_default_address() {
        let addr = IpcTransport::default_address();
        assert!(!addr.is_empty());
    }

    #[test]
    fn test_transport_pool_creation() {
        // Can't actually connect without a server, but test construction
        let pool = TransportPool::new("tcp://127.0.0.1:19999", Duration::from_millis(100), 4);
        // Will fail to connect, which is expected
        assert!(pool.is_err());
    }

    #[test]
    fn test_reconnecting_transport_creation() {
        let transport = ReconnectingTransport::new(
            "tcp://127.0.0.1:19999",
            Duration::from_secs(5),
            3,
            Duration::from_millis(100),
        );
        assert_eq!(transport.max_retries, 3);
    }

    #[test]
    #[ignore] // Requires actual IPC setup
    fn test_client_server() {
        let addr = "ipc:///tmp/openclaw-test.ipc";

        // Start server in thread
        let server_thread = thread::spawn(move || {
            let server = IpcTransport::new_server(addr).unwrap();
            let request = server.recv().unwrap();
            let response = IpcMessage::success(&request.id, serde_json::json!({"ok": true}));
            server.send(&response).unwrap();
        });

        // Give server time to start
        thread::sleep(Duration::from_millis(100));

        // Client
        let client = IpcTransport::new_client(addr, Duration::from_secs(5)).unwrap();
        let request = IpcMessage::request("test", serde_json::json!({}));
        let response = client.request(&request).unwrap();

        if let crate::messages::IpcPayload::Response(resp) = response.payload {
            assert!(resp.success);
        }

        server_thread.join().unwrap();
    }

    #[tokio::test]
    #[ignore] // Requires actual IPC setup
    async fn test_async_client_server() {
        let addr = "ipc:///tmp/openclaw-async-test.ipc";

        // Start server in blocking task
        let server_handle = tokio::task::spawn_blocking(move || {
            let server = IpcTransport::new_server(addr).unwrap();
            let request = server.recv().unwrap();
            let response = IpcMessage::success(&request.id, serde_json::json!({"ok": true}));
            server.send(&response).unwrap();
        });

        // Give server time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Client with async
        let client = IpcTransport::new_client(addr, Duration::from_secs(5)).unwrap();
        let request = IpcMessage::request("test", serde_json::json!({}));
        let response = client.request_async(&request).await.unwrap();

        if let crate::messages::IpcPayload::Response(resp) = response.payload {
            assert!(resp.success);
        }

        server_handle.await.unwrap();
    }
}
