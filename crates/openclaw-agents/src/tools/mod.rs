//! Tool registry and execution.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use openclaw_providers::traits::Tool as ToolDefinition;

/// Tool execution errors.
#[derive(Error, Debug)]
pub enum ToolError {
    /// Tool not found.
    #[error("Tool not found: {0}")]
    NotFound(String),

    /// Invalid parameters.
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    /// Execution failed.
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    /// Tool timed out.
    #[error("Tool timed out")]
    Timeout,
}

/// Tool execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether execution succeeded.
    pub success: bool,
    /// Result content.
    pub content: String,
    /// Error message if failed.
    pub error: Option<String>,
}

impl ToolResult {
    /// Create a successful result.
    #[must_use]
    pub fn success(content: impl Into<String>) -> Self {
        Self {
            success: true,
            content: content.into(),
            error: None,
        }
    }

    /// Create an error result.
    #[must_use]
    pub fn error(error: impl Into<String>) -> Self {
        Self {
            success: false,
            content: String::new(),
            error: Some(error.into()),
        }
    }
}

/// Tool trait for implementing custom tools.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool name.
    fn name(&self) -> &str;

    /// Tool description.
    fn description(&self) -> &str;

    /// Input schema (JSON Schema).
    fn input_schema(&self) -> serde_json::Value;

    /// Execute the tool.
    async fn execute(&self, params: serde_json::Value) -> Result<ToolResult, ToolError>;
}

/// Registry of available tools.
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool.
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Get a tool by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Arc<dyn Tool>> {
        self.tools.get(name)
    }

    /// List all tool names.
    #[must_use]
    pub fn list(&self) -> Vec<&str> {
        self.tools.keys().map(String::as_str).collect()
    }

    /// Execute a tool by name.
    ///
    /// # Errors
    ///
    /// Returns error if tool not found or execution fails.
    pub async fn execute(
        &self,
        name: &str,
        params: serde_json::Value,
    ) -> Result<ToolResult, ToolError> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| ToolError::NotFound(name.to_string()))?;
        tool.execute(params).await
    }

    /// Get tool definitions for provider API.
    #[must_use]
    pub fn as_tool_definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|tool| ToolDefinition {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                input_schema: tool.input_schema(),
            })
            .collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in bash tool for command execution.
pub struct BashTool {
    sandbox_config: crate::sandbox::SandboxConfig,
}

impl BashTool {
    /// Create a new bash tool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sandbox_config: crate::sandbox::SandboxConfig::default(),
        }
    }

    /// Create with custom sandbox config.
    #[must_use]
    pub fn with_sandbox_config(config: crate::sandbox::SandboxConfig) -> Self {
        Self {
            sandbox_config: config,
        }
    }
}

impl Default for BashTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Execute a bash command in a sandboxed environment"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The bash command to execute"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, params: serde_json::Value) -> Result<ToolResult, ToolError> {
        let command = params["command"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidParams("Missing 'command' parameter".to_string()))?;

        // Execute in sandbox
        let output =
            crate::sandbox::execute_sandboxed("bash", &["-c", command], &self.sandbox_config)
                .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        if output.exit_code == 0 {
            Ok(ToolResult::success(output.stdout))
        } else {
            let error_msg = if output.stderr.is_empty() {
                format!("Command failed with exit code {}", output.exit_code)
            } else {
                output.stderr
            };
            Ok(ToolResult::error(error_msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(BashTool::new()));

        assert!(registry.get("bash").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_tool_definitions() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(BashTool::new()));

        let defs = registry.as_tool_definitions();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "bash");
    }
}
