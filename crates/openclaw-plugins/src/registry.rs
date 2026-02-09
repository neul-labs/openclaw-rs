//! Plugin registry.

use std::collections::HashMap;
use std::sync::Arc;

use crate::api::{Plugin, PluginError, PluginHook};

/// Registry of loaded plugins.
pub struct PluginRegistry {
    plugins: HashMap<String, Arc<dyn Plugin>>,
}

impl PluginRegistry {
    /// Create a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Register a plugin.
    pub fn register(&mut self, plugin: Arc<dyn Plugin>) {
        self.plugins.insert(plugin.id().to_string(), plugin);
    }

    /// Get a plugin by ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Arc<dyn Plugin>> {
        self.plugins.get(id)
    }

    /// List all plugin IDs.
    #[must_use]
    pub fn list(&self) -> Vec<&str> {
        self.plugins.keys().map(String::as_str).collect()
    }

    /// Execute a hook on all plugins that support it.
    pub async fn execute_hook(
        &self,
        hook: PluginHook,
        data: serde_json::Value,
    ) -> Vec<Result<serde_json::Value, PluginError>> {
        let mut results = Vec::new();

        for plugin in self.plugins.values() {
            if plugin.hooks().contains(&hook) {
                results.push(plugin.execute_hook(hook, data.clone()).await);
            }
        }

        results
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
