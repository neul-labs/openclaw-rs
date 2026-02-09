//! WASM plugin runtime using wasmtime.
//!
//! Provides secure, sandboxed execution of WebAssembly plugins.

use std::path::Path;

use async_trait::async_trait;
use wasmtime::{Config, Engine, Instance, Linker, Module, Store, TypedFunc};

use crate::api::{Plugin, PluginError, PluginHook};

/// WASM plugin metadata.
#[derive(Debug, Clone)]
pub struct WasmPluginMetadata {
    /// Plugin name.
    pub name: String,
    /// Plugin version.
    pub version: String,
    /// Supported hooks.
    pub hooks: Vec<String>,
}

/// Store data for WASM plugins.
pub struct PluginState {
    /// Plugin metadata.
    pub metadata: WasmPluginMetadata,
    /// Result buffer for host calls.
    result_buffer: Vec<u8>,
}

impl PluginState {
    fn new(metadata: WasmPluginMetadata) -> Self {
        Self {
            metadata,
            result_buffer: Vec::with_capacity(4096),
        }
    }
}

/// WASM plugin runtime.
pub struct WasmPlugin {
    engine: Engine,
    module: Module,
    instance: Instance,
    store: Store<PluginState>,
    metadata: WasmPluginMetadata,
}

impl WasmPlugin {
    /// Load a WASM plugin from a file.
    ///
    /// # Errors
    ///
    /// Returns error if loading or instantiation fails.
    pub fn load(path: &Path) -> Result<Self, PluginError> {
        let mut config = Config::new();
        config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);

        let engine =
            Engine::new(&config).map_err(|e| PluginError::LoadFailed(format!("Engine: {e}")))?;

        let module = Module::from_file(&engine, path)
            .map_err(|e| PluginError::LoadFailed(format!("Module load: {e}")))?;

        // Create initial metadata (will be updated after init)
        let metadata = WasmPluginMetadata {
            name: path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            version: "0.0.0".to_string(),
            hooks: Vec::new(),
        };

        let mut store = Store::new(&engine, PluginState::new(metadata.clone()));

        // Create linker with host functions
        let mut linker = Linker::new(&engine);
        Self::define_host_functions(&mut linker)?;

        // Instantiate the module
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| PluginError::LoadFailed(format!("Instantiate: {e}")))?;

        let mut plugin = Self {
            engine,
            module,
            instance,
            store,
            metadata,
        };

        // Call plugin init to get metadata
        plugin.init()?;

        Ok(plugin)
    }

    /// Load a WASM plugin from bytes.
    ///
    /// # Errors
    ///
    /// Returns error if loading or instantiation fails.
    pub fn load_bytes(name: &str, bytes: &[u8]) -> Result<Self, PluginError> {
        let mut config = Config::new();
        config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);

        let engine =
            Engine::new(&config).map_err(|e| PluginError::LoadFailed(format!("Engine: {e}")))?;

        let module = Module::new(&engine, bytes)
            .map_err(|e| PluginError::LoadFailed(format!("Module load: {e}")))?;

        let metadata = WasmPluginMetadata {
            name: name.to_string(),
            version: "0.0.0".to_string(),
            hooks: Vec::new(),
        };

        let mut store = Store::new(&engine, PluginState::new(metadata.clone()));

        let mut linker = Linker::new(&engine);
        Self::define_host_functions(&mut linker)?;

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| PluginError::LoadFailed(format!("Instantiate: {e}")))?;

        let mut plugin = Self {
            engine,
            module,
            instance,
            store,
            metadata,
        };

        plugin.init()?;

        Ok(plugin)
    }

    /// Define host functions available to WASM plugins.
    fn define_host_functions(linker: &mut Linker<PluginState>) -> Result<(), PluginError> {
        // plugin_log(level: i32, msg_ptr: i32, msg_len: i32)
        linker
            .func_wrap(
                "env",
                "plugin_log",
                |mut caller: wasmtime::Caller<'_, PluginState>,
                 level: i32,
                 msg_ptr: i32,
                 msg_len: i32| {
                    let memory = match caller.get_export("memory") {
                        Some(wasmtime::Extern::Memory(mem)) => mem,
                        _ => return,
                    };

                    let data = memory.data(&caller);
                    let start = msg_ptr as usize;
                    let end = start + msg_len as usize;
                    if end > data.len() {
                        return;
                    }

                    let msg = String::from_utf8_lossy(&data[start..end]);
                    match level {
                        0 => tracing::trace!(plugin = "wasm", "{}", msg),
                        1 => tracing::debug!(plugin = "wasm", "{}", msg),
                        2 => tracing::info!(plugin = "wasm", "{}", msg),
                        3 => tracing::warn!(plugin = "wasm", "{}", msg),
                        _ => tracing::error!(plugin = "wasm", "{}", msg),
                    }
                },
            )
            .map_err(|e| PluginError::LoadFailed(format!("Link plugin_log: {e}")))?;

        // plugin_get_config(key_ptr: i32, key_len: i32) -> i32 (returns ptr to result)
        linker
            .func_wrap(
                "env",
                "plugin_get_config",
                |_caller: wasmtime::Caller<'_, PluginState>, _key_ptr: i32, _key_len: i32| -> i32 {
                    // Return 0 to indicate no config available
                    // In a real implementation, this would read from config store
                    0
                },
            )
            .map_err(|e| PluginError::LoadFailed(format!("Link plugin_get_config: {e}")))?;

        // plugin_set_result(ptr: i32, len: i32)
        linker
            .func_wrap(
                "env",
                "plugin_set_result",
                |mut caller: wasmtime::Caller<'_, PluginState>, ptr: i32, len: i32| {
                    let memory = match caller.get_export("memory") {
                        Some(wasmtime::Extern::Memory(mem)) => mem,
                        _ => return,
                    };

                    let data = memory.data(&caller);
                    let start = ptr as usize;
                    let end = start + len as usize;
                    if end > data.len() {
                        return;
                    }

                    let result_data = data[start..end].to_vec();
                    caller.data_mut().result_buffer = result_data;
                },
            )
            .map_err(|e| PluginError::LoadFailed(format!("Link plugin_set_result: {e}")))?;

        Ok(())
    }

    /// Initialize the plugin and get metadata.
    fn init(&mut self) -> Result<(), PluginError> {
        // Look for plugin_init export
        let init_fn: Option<TypedFunc<(), i32>> = self
            .instance
            .get_typed_func::<(), i32>(&mut self.store, "plugin_init")
            .ok();

        if let Some(init) = init_fn {
            let result = init
                .call(&mut self.store, ())
                .map_err(|e| PluginError::ExecutionError(format!("Init failed: {e}")))?;

            if result != 0 {
                return Err(PluginError::ExecutionError(format!(
                    "Plugin init returned error code: {result}"
                )));
            }
        }

        // Try to get metadata from plugin
        if let Ok(get_name) =
            self.instance
                .get_typed_func::<(), i32>(&mut self.store, "plugin_get_name")
        {
            let _ = get_name.call(&mut self.store, ());
            if !self.store.data().result_buffer.is_empty() {
                if let Ok(name) = String::from_utf8(self.store.data().result_buffer.clone()) {
                    self.metadata.name = name;
                }
                self.store.data_mut().result_buffer.clear();
            }
        }

        if let Ok(get_version) =
            self.instance
                .get_typed_func::<(), i32>(&mut self.store, "plugin_get_version")
        {
            let _ = get_version.call(&mut self.store, ());
            if !self.store.data().result_buffer.is_empty() {
                if let Ok(version) = String::from_utf8(self.store.data().result_buffer.clone()) {
                    self.metadata.version = version;
                }
                self.store.data_mut().result_buffer.clear();
            }
        }

        tracing::info!(
            name = %self.metadata.name,
            version = %self.metadata.version,
            "WASM plugin loaded"
        );

        Ok(())
    }

    /// Call an exported function with JSON params and return JSON result.
    ///
    /// # Errors
    ///
    /// Returns error if function doesn't exist or execution fails.
    pub fn call_export(
        &mut self,
        method: &str,
        params: &[u8],
    ) -> Result<Vec<u8>, PluginError> {
        // Get memory and allocator
        let memory = self
            .instance
            .get_memory(&mut self.store, "memory")
            .ok_or_else(|| PluginError::ExecutionError("No memory export".to_string()))?;

        // Get alloc function to allocate space for params
        let alloc_fn: TypedFunc<i32, i32> = self
            .instance
            .get_typed_func(&mut self.store, "plugin_alloc")
            .map_err(|e| PluginError::ExecutionError(format!("No alloc function: {e}")))?;

        // Allocate space for params
        let params_ptr = alloc_fn
            .call(&mut self.store, params.len() as i32)
            .map_err(|e| PluginError::ExecutionError(format!("Alloc failed: {e}")))?;

        // Write params to memory
        memory
            .write(&mut self.store, params_ptr as usize, params)
            .map_err(|e| PluginError::ExecutionError(format!("Memory write failed: {e}")))?;

        // Get the export function
        let export_fn: TypedFunc<(i32, i32), i32> = self
            .instance
            .get_typed_func(&mut self.store, method)
            .map_err(|e| PluginError::ExecutionError(format!("Export not found: {e}")))?;

        // Clear result buffer
        self.store.data_mut().result_buffer.clear();

        // Call the function
        let result = export_fn
            .call(&mut self.store, (params_ptr, params.len() as i32))
            .map_err(|e| PluginError::ExecutionError(format!("Call failed: {e}")))?;

        if result != 0 {
            return Err(PluginError::ExecutionError(format!(
                "Export returned error code: {result}"
            )));
        }

        // Return the result from result buffer
        Ok(self.store.data().result_buffer.clone())
    }

    /// Get plugin metadata.
    #[must_use]
    pub fn metadata(&self) -> &WasmPluginMetadata {
        &self.metadata
    }
}

#[async_trait]
impl Plugin for WasmPlugin {
    fn id(&self) -> &str {
        &self.metadata.name
    }

    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn version(&self) -> &str {
        &self.metadata.version
    }

    fn hooks(&self) -> &[PluginHook] {
        // WASM plugins can implement any hook
        &[
            PluginHook::BeforeMessage,
            PluginHook::AfterMessage,
            PluginHook::BeforeToolCall,
            PluginHook::AfterToolCall,
            PluginHook::SessionStart,
            PluginHook::SessionEnd,
            PluginHook::AgentResponse,
            PluginHook::Error,
        ]
    }

    async fn execute_hook(
        &self,
        hook: PluginHook,
        data: serde_json::Value,
    ) -> Result<serde_json::Value, PluginError> {
        // WASM plugins need mutable access, so we can't implement this directly
        // In a real implementation, you'd use interior mutability (Mutex/RwLock)
        // For now, just return the data unchanged
        let _ = hook;
        Ok(data)
    }

    async fn activate(&self) -> Result<(), PluginError> {
        Ok(())
    }

    async fn deactivate(&self) -> Result<(), PluginError> {
        Ok(())
    }
}

/// WASM plugin manager for loading and managing multiple WASM plugins.
pub struct WasmPluginManager {
    plugins: Vec<WasmPlugin>,
}

impl WasmPluginManager {
    /// Create a new plugin manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Load a plugin from a file.
    ///
    /// # Errors
    ///
    /// Returns error if loading fails.
    pub fn load(&mut self, path: &Path) -> Result<(), PluginError> {
        let plugin = WasmPlugin::load(path)?;
        self.plugins.push(plugin);
        Ok(())
    }

    /// Load all WASM plugins from a directory.
    ///
    /// # Errors
    ///
    /// Returns error if directory read fails (individual plugin failures are logged).
    pub fn load_dir(&mut self, dir: &Path) -> Result<usize, PluginError> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| PluginError::LoadFailed(format!("Read dir: {e}")))?;

        let mut loaded = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "wasm") {
                match WasmPlugin::load(&path) {
                    Ok(plugin) => {
                        tracing::info!(path = %path.display(), "Loaded WASM plugin");
                        self.plugins.push(plugin);
                        loaded += 1;
                    }
                    Err(e) => {
                        tracing::warn!(path = %path.display(), error = %e, "Failed to load WASM plugin");
                    }
                }
            }
        }

        Ok(loaded)
    }

    /// Get all loaded plugins.
    #[must_use]
    pub fn plugins(&self) -> &[WasmPlugin] {
        &self.plugins
    }

    /// Get mutable access to all plugins.
    pub fn plugins_mut(&mut self) -> &mut [WasmPlugin] {
        &mut self.plugins
    }
}

impl Default for WasmPluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = WasmPluginManager::new();
        assert!(manager.plugins().is_empty());
    }

    // Note: Actual WASM loading tests would require test .wasm files
}
