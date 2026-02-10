//! Native plugin support via dynamic library loading.
//!
//! Provides FFI-based plugin loading for high-performance native plugins.
//!
//! # Safety
//!
//! This module uses unsafe code for FFI with native plugins.
//! Only load plugins from trusted sources.

#![allow(unsafe_code)]

use std::ffi::{CStr, c_char, c_int};
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use libloading::{Library, Symbol};

use crate::api::{Plugin, PluginError, PluginHook};

/// Plugin API version for ABI compatibility.
pub const PLUGIN_API_VERSION: u32 = 1;

/// Plugin info returned by native plugins.
#[repr(C)]
pub struct CPluginInfo {
    /// API version (must match PLUGIN_API_VERSION).
    pub api_version: u32,
    /// Plugin name (null-terminated UTF-8).
    pub name: *const c_char,
    /// Plugin version string (null-terminated UTF-8).
    pub version: *const c_char,
}

/// Result from hook execution.
#[repr(C)]
pub struct CHookResult {
    /// Success flag (0 = success, non-zero = error).
    pub status: c_int,
    /// Result data pointer (owned by plugin).
    pub data: *const u8,
    /// Result data length.
    pub data_len: usize,
    /// Error message if status != 0 (null-terminated UTF-8).
    pub error: *const c_char,
}

/// Type alias for plugin_get_info function.
type GetInfoFn = unsafe extern "C" fn() -> *const CPluginInfo;
/// Type alias for plugin_init function.
type InitFn = unsafe extern "C" fn() -> c_int;
/// Type alias for plugin_deinit function.
type DeinitFn = unsafe extern "C" fn() -> c_int;
/// Type alias for plugin_execute_hook function.
type ExecuteHookFn =
    unsafe extern "C" fn(hook_id: c_int, data: *const u8, data_len: usize) -> CHookResult;
/// Type alias for plugin_free_result function.
type FreeResultFn = unsafe extern "C" fn(result: *mut CHookResult);

/// Native plugin loaded from a dynamic library.
pub struct NativePlugin {
    #[allow(dead_code)]
    library: Library,
    info: NativePluginInfo,
    init_fn: Option<InitFn>,
    deinit_fn: Option<DeinitFn>,
    execute_hook_fn: Option<ExecuteHookFn>,
    free_result_fn: Option<FreeResultFn>,
    initialized: bool,
}

/// Native plugin metadata.
#[derive(Debug, Clone)]
pub struct NativePluginInfo {
    /// Plugin name.
    pub name: String,
    /// Plugin version.
    pub version: String,
    /// Path to the library.
    pub path: PathBuf,
}

impl NativePlugin {
    /// Load a native plugin from a dynamic library.
    ///
    /// # Safety
    ///
    /// This function loads and executes code from the specified library.
    /// Only load libraries from trusted sources.
    ///
    /// # Errors
    ///
    /// Returns error if loading fails or ABI version mismatch.
    pub fn load(path: &Path) -> Result<Self, PluginError> {
        // Load the library
        let library = unsafe {
            Library::new(path).map_err(|e| {
                PluginError::LoadFailed(format!("Failed to load library {}: {e}", path.display()))
            })?
        };

        // Get plugin_get_info (required)
        let get_info: Symbol<GetInfoFn> = unsafe {
            library.get(b"plugin_get_info").map_err(|e| {
                PluginError::LoadFailed(format!("Missing plugin_get_info export: {e}"))
            })?
        };

        // Call get_info to verify API version
        let c_info = unsafe { get_info() };
        if c_info.is_null() {
            return Err(PluginError::LoadFailed(
                "plugin_get_info returned null".to_string(),
            ));
        }

        let c_info = unsafe { &*c_info };

        // Check API version
        if c_info.api_version != PLUGIN_API_VERSION {
            return Err(PluginError::LoadFailed(format!(
                "API version mismatch: expected {}, got {}",
                PLUGIN_API_VERSION, c_info.api_version
            )));
        }

        // Extract plugin info
        let name = if c_info.name.is_null() {
            "unknown".to_string()
        } else {
            unsafe {
                CStr::from_ptr(c_info.name)
                    .to_str()
                    .unwrap_or("unknown")
                    .to_string()
            }
        };

        let version = if c_info.version.is_null() {
            "0.0.0".to_string()
        } else {
            unsafe {
                CStr::from_ptr(c_info.version)
                    .to_str()
                    .unwrap_or("0.0.0")
                    .to_string()
            }
        };

        let info = NativePluginInfo {
            name,
            version,
            path: path.to_path_buf(),
        };

        // Get optional functions
        let init_fn: Option<InitFn> = unsafe { library.get(b"plugin_init").ok().map(|s| *s) };

        let deinit_fn: Option<DeinitFn> = unsafe { library.get(b"plugin_deinit").ok().map(|s| *s) };

        let execute_hook_fn: Option<ExecuteHookFn> =
            unsafe { library.get(b"plugin_execute_hook").ok().map(|s| *s) };

        let free_result_fn: Option<FreeResultFn> =
            unsafe { library.get(b"plugin_free_result").ok().map(|s| *s) };

        let mut plugin = Self {
            library,
            info,
            init_fn,
            deinit_fn,
            execute_hook_fn,
            free_result_fn,
            initialized: false,
        };

        // Initialize the plugin
        plugin.init()?;

        Ok(plugin)
    }

    /// Initialize the plugin.
    fn init(&mut self) -> Result<(), PluginError> {
        if self.initialized {
            return Ok(());
        }

        if let Some(init) = self.init_fn {
            let result = unsafe { init() };
            if result != 0 {
                return Err(PluginError::ExecutionError(format!(
                    "Plugin init failed with code: {result}"
                )));
            }
        }

        self.initialized = true;
        tracing::info!(
            name = %self.info.name,
            version = %self.info.version,
            "Native plugin loaded"
        );

        Ok(())
    }

    /// Execute a hook.
    fn execute_hook_internal(&self, hook_id: i32, data: &[u8]) -> Result<Vec<u8>, PluginError> {
        let execute = self
            .execute_hook_fn
            .ok_or_else(|| PluginError::ExecutionError("No execute_hook export".to_string()))?;

        let result = unsafe { execute(hook_id, data.as_ptr(), data.len()) };

        if result.status != 0 {
            let error_msg = if result.error.is_null() {
                format!("Hook execution failed with code: {}", result.status)
            } else {
                unsafe {
                    CStr::from_ptr(result.error)
                        .to_str()
                        .unwrap_or("Unknown error")
                        .to_string()
                }
            };

            // Free the result if needed
            if let Some(free_fn) = self.free_result_fn {
                unsafe { free_fn(&result as *const _ as *mut _) };
            }

            return Err(PluginError::ExecutionError(error_msg));
        }

        // Copy result data
        let result_data = if result.data.is_null() || result.data_len == 0 {
            Vec::new()
        } else {
            unsafe { std::slice::from_raw_parts(result.data, result.data_len).to_vec() }
        };

        // Free the result
        if let Some(free_fn) = self.free_result_fn {
            unsafe { free_fn(&result as *const _ as *mut _) };
        }

        Ok(result_data)
    }

    /// Get plugin info.
    #[must_use]
    pub fn info(&self) -> &NativePluginInfo {
        &self.info
    }
}

impl Drop for NativePlugin {
    fn drop(&mut self) {
        if self.initialized {
            if let Some(deinit) = self.deinit_fn {
                let result = unsafe { deinit() };
                if result != 0 {
                    tracing::warn!(
                        plugin = %self.info.name,
                        code = result,
                        "Plugin deinit returned error"
                    );
                }
            }
        }
    }
}

#[async_trait]
impl Plugin for NativePlugin {
    fn id(&self) -> &str {
        &self.info.name
    }

    fn name(&self) -> &str {
        &self.info.name
    }

    fn version(&self) -> &str {
        &self.info.version
    }

    fn hooks(&self) -> &[PluginHook] {
        // Native plugins can implement any hook
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
        let hook_id = match hook {
            PluginHook::BeforeMessage => 0,
            PluginHook::AfterMessage => 1,
            PluginHook::BeforeToolCall => 2,
            PluginHook::AfterToolCall => 3,
            PluginHook::SessionStart => 4,
            PluginHook::SessionEnd => 5,
            PluginHook::AgentResponse => 6,
            PluginHook::Error => 7,
        };

        let input = serde_json::to_vec(&data)
            .map_err(|e| PluginError::ExecutionError(format!("Serialize: {e}")))?;

        let output = self.execute_hook_internal(hook_id, &input)?;

        if output.is_empty() {
            return Ok(data);
        }

        serde_json::from_slice(&output)
            .map_err(|e| PluginError::ExecutionError(format!("Deserialize: {e}")))
    }

    async fn activate(&self) -> Result<(), PluginError> {
        Ok(())
    }

    async fn deactivate(&self) -> Result<(), PluginError> {
        Ok(())
    }
}

/// Discover native plugins in a directory.
///
/// Looks for platform-appropriate shared libraries (.so, .dylib, .dll).
pub fn discover_native_plugins(dir: &Path) -> Vec<PathBuf> {
    let extension = if cfg!(windows) {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };

    let mut plugins = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == extension) {
                plugins.push(path);
            }
        }
    }

    plugins
}

/// Native plugin manager for loading and managing native plugins.
pub struct NativePluginManager {
    plugins: Vec<NativePlugin>,
}

impl NativePluginManager {
    /// Create a new plugin manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Load a plugin from a library file.
    ///
    /// # Errors
    ///
    /// Returns error if loading fails.
    pub fn load(&mut self, path: &Path) -> Result<(), PluginError> {
        let plugin = NativePlugin::load(path)?;
        self.plugins.push(plugin);
        Ok(())
    }

    /// Load all native plugins from a directory.
    ///
    /// # Errors
    ///
    /// Returns error if directory read fails (individual failures are logged).
    pub fn load_dir(&mut self, dir: &Path) -> Result<usize, PluginError> {
        let paths = discover_native_plugins(dir);
        let mut loaded = 0;

        for path in paths {
            match NativePlugin::load(&path) {
                Ok(plugin) => {
                    tracing::info!(path = %path.display(), "Loaded native plugin");
                    self.plugins.push(plugin);
                    loaded += 1;
                }
                Err(e) => {
                    tracing::warn!(path = %path.display(), error = %e, "Failed to load native plugin");
                }
            }
        }

        Ok(loaded)
    }

    /// Get all loaded plugins.
    #[must_use]
    pub fn plugins(&self) -> &[NativePlugin] {
        &self.plugins
    }
}

impl Default for NativePluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_version() {
        assert_eq!(PLUGIN_API_VERSION, 1);
    }

    #[test]
    fn test_manager_creation() {
        let manager = NativePluginManager::new();
        assert!(manager.plugins().is_empty());
    }

    #[test]
    fn test_discover_empty_dir() {
        let dir = std::env::temp_dir().join("openclaw-test-empty");
        let _ = std::fs::create_dir_all(&dir);
        let plugins = discover_native_plugins(&dir);
        assert!(plugins.is_empty());
    }
}
