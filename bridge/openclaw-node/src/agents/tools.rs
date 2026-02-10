//! Tool registry and execution bindings.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::OpenClawError;

/// Result of a tool execution.
#[napi(object)]
#[derive(Debug, Clone)]
pub struct JsToolResult {
    /// Whether the tool executed successfully
    pub success: bool,
    /// Tool output content
    pub content: String,
    /// Error message (if success is false)
    pub error: Option<String>,
}

impl JsToolResult {
    /// Create a successful result.
    pub fn success(content: impl Into<String>) -> Self {
        Self {
            success: true,
            content: content.into(),
            error: None,
        }
    }

    /// Create an error result.
    pub fn error(message: impl Into<String>) -> Self {
        let msg = message.into();
        Self {
            success: false,
            content: String::new(),
            error: Some(msg),
        }
    }
}

/// A tool definition (for listing/inspection).
#[napi(object)]
#[derive(Debug, Clone)]
pub struct JsToolDefinition {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// JSON schema for input parameters
    pub input_schema: serde_json::Value,
}

/// Internal tool storage.
struct StoredTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
    // We store the threadsafe function for execution
    execute_fn: Option<napi::threadsafe_function::ThreadsafeFunction<serde_json::Value, napi::threadsafe_function::ErrorStrategy::Fatal>>,
}

/// Tool registry for managing and executing tools.
///
/// Tools can be registered from JavaScript and then executed by name.
///
/// ```javascript
/// const registry = new ToolRegistry();
///
/// // Register a tool with a callback
/// registry.registerCallback('echo', 'Echoes input', schema, async (params) => {
///   return { success: true, content: params.message, error: null };
/// });
///
/// // Execute the tool
/// const result = await registry.execute('echo', { message: 'hello' });
/// ```
#[napi]
pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, StoredTool>>>,
}

#[napi]
impl ToolRegistry {
    /// Create a new empty tool registry.
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a tool with a JavaScript callback.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique tool name
    /// * `description` - Human-readable description
    /// * `input_schema` - JSON schema for parameters
    /// * `execute_fn` - Async function that takes params and returns JsToolResult
    #[napi]
    pub fn register_callback(
        &self,
        name: String,
        description: String,
        input_schema: serde_json::Value,
        #[napi(ts_arg_type = "(params: any) => Promise<JsToolResult>")]
        execute_fn: JsFunction,
    ) -> Result<()> {
        use napi::threadsafe_function::{ThreadsafeFunction, ErrorStrategy};

        let tsfn: ThreadsafeFunction<serde_json::Value, ErrorStrategy::Fatal> =
            execute_fn.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

        let tool = StoredTool {
            name: name.clone(),
            description,
            input_schema,
            execute_fn: Some(tsfn),
        };

        // Use blocking to insert into the map
        let tools = self.tools.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut guard = tools.write().await;
                guard.insert(name, tool);
            });
        }).join().map_err(|_| OpenClawError::tool_error("Failed to register tool"))?;

        Ok(())
    }

    /// Register a tool definition without a callback.
    ///
    /// Useful for documenting tools that will be executed externally.
    #[napi]
    pub async fn register_definition(
        &self,
        name: String,
        description: String,
        input_schema: serde_json::Value,
    ) -> Result<()> {
        let tool = StoredTool {
            name: name.clone(),
            description,
            input_schema,
            execute_fn: None,
        };

        let mut tools = self.tools.write().await;
        tools.insert(name, tool);
        Ok(())
    }

    /// Unregister a tool by name.
    #[napi]
    pub async fn unregister(&self, name: String) -> Result<bool> {
        let mut tools = self.tools.write().await;
        Ok(tools.remove(&name).is_some())
    }

    /// List all registered tool names.
    #[napi]
    pub async fn list(&self) -> Vec<String> {
        let tools = self.tools.read().await;
        tools.keys().cloned().collect()
    }

    /// Get a tool definition by name.
    #[napi]
    pub async fn get(&self, name: String) -> Option<JsToolDefinition> {
        let tools = self.tools.read().await;
        tools.get(&name).map(|t| JsToolDefinition {
            name: t.name.clone(),
            description: t.description.clone(),
            input_schema: t.input_schema.clone(),
        })
    }

    /// Get all tool definitions.
    #[napi]
    pub async fn get_all(&self) -> Vec<JsToolDefinition> {
        let tools = self.tools.read().await;
        tools.values().map(|t| JsToolDefinition {
            name: t.name.clone(),
            description: t.description.clone(),
            input_schema: t.input_schema.clone(),
        }).collect()
    }

    /// Check if a tool is registered.
    #[napi]
    pub async fn has(&self, name: String) -> bool {
        let tools = self.tools.read().await;
        tools.contains_key(&name)
    }

    /// Get the number of registered tools.
    #[napi]
    pub async fn count(&self) -> u32 {
        let tools = self.tools.read().await;
        tools.len() as u32
    }

    /// Clear all registered tools.
    #[napi]
    pub async fn clear(&self) {
        let mut tools = self.tools.write().await;
        tools.clear();
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
