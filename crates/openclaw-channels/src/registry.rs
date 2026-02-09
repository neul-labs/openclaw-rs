//! Channel registry.

use std::collections::HashMap;
use std::sync::Arc;

use crate::traits::{Channel, ChannelError, ChannelProbe};

/// Registry of available channels.
pub struct ChannelRegistry {
    channels: HashMap<String, Arc<dyn Channel>>,
}

impl ChannelRegistry {
    /// Create a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
        }
    }

    /// Register a channel.
    pub fn register(&mut self, channel: Arc<dyn Channel>) {
        self.channels.insert(channel.id().to_string(), channel);
    }

    /// Get a channel by ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Arc<dyn Channel>> {
        self.channels.get(id)
    }

    /// List all channel IDs.
    #[must_use]
    pub fn list(&self) -> Vec<&str> {
        self.channels.keys().map(String::as_str).collect()
    }

    /// Probe all channels.
    pub async fn probe_all(&self) -> HashMap<String, Result<ChannelProbe, ChannelError>> {
        let mut results = HashMap::new();
        for (id, channel) in &self.channels {
            results.insert(id.clone(), channel.probe().await);
        }
        results
    }
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        Self::new()
    }
}
