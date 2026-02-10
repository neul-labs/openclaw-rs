//! Plugin API.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Plugin errors.
#[derive(Error, Debug)]
pub enum PluginError {
    /// Plugin not found.
    #[error("Plugin not found: {0}")]
    NotFound(String),

    /// Plugin load failed.
    #[error("Load failed: {0}")]
    LoadFailed(String),

    /// Hook execution failed.
    #[error("Hook failed: {0}")]
    HookFailed(String),

    /// General execution error.
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// IPC error.
    #[error("IPC error: {0}")]
    Ipc(String),
}

/// Plugin lifecycle hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PluginHook {
    /// Before agent processes message.
    BeforeMessage,
    /// After agent processes message.
    AfterMessage,
    /// Before tool execution.
    BeforeToolCall,
    /// After tool execution.
    AfterToolCall,
    /// Session started.
    SessionStart,
    /// Session ended.
    SessionEnd,
    /// Agent response generated.
    AgentResponse,
    /// Error occurred.
    Error,
}

/// Plugin trait.
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Plugin ID.
    fn id(&self) -> &str;

    /// Plugin name.
    fn name(&self) -> &str;

    /// Plugin version.
    fn version(&self) -> &str;

    /// Supported hooks.
    fn hooks(&self) -> &[PluginHook];

    /// Execute a hook.
    async fn execute_hook(
        &self,
        hook: PluginHook,
        data: serde_json::Value,
    ) -> Result<serde_json::Value, PluginError>;

    /// Activate the plugin.
    async fn activate(&self) -> Result<(), PluginError>;

    /// Deactivate the plugin.
    async fn deactivate(&self) -> Result<(), PluginError>;
}

/// Plugin API exposed to plugins.
pub struct PluginApi {
    // Context available to plugins
}

impl PluginApi {
    /// Create a new plugin API.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for PluginApi {
    fn default() -> Self {
        Self::new()
    }
}
