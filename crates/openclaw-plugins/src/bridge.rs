//! TypeScript plugin bridge via IPC.

use std::path::{Path, PathBuf};
use std::process::Child;

use async_trait::async_trait;
use openclaw_ipc::{IpcMessage, IpcTransport};

use crate::api::{Plugin, PluginError, PluginHook};

/// Bridge to existing TypeScript plugins.
pub struct TsPluginBridge {
    transport: Option<IpcTransport>,
    plugins_dir: PathBuf,
    child_process: Option<Child>,
    ipc_address: String,
    manifest: Option<SkillManifest>,
}

impl TsPluginBridge {
    /// Create a new bridge (not connected).
    #[must_use]
    pub fn new(plugins_dir: &Path) -> Self {
        Self {
            transport: None,
            plugins_dir: plugins_dir.to_path_buf(),
            child_process: None,
            ipc_address: IpcTransport::default_address(),
            manifest: None,
        }
    }

    /// Set a custom IPC address.
    #[must_use]
    pub fn with_address(mut self, address: impl Into<String>) -> Self {
        self.ipc_address = address.into();
        self
    }

    /// Connect to an already-running TypeScript plugin host.
    ///
    /// # Errors
    ///
    /// Returns error if connection fails.
    pub fn connect(&mut self, address: &str) -> Result<(), PluginError> {
        let transport = IpcTransport::new_client(address, std::time::Duration::from_secs(30))
            .map_err(|e| PluginError::Ipc(e.to_string()))?;
        self.transport = Some(transport);
        Ok(())
    }

    /// Spawn the TypeScript plugin host process and connect.
    ///
    /// This looks for a `plugin-host.js` or `plugin-host.ts` entry point
    /// in the plugins directory and runs it with Node.js or Bun.
    ///
    /// # Errors
    ///
    /// Returns error if process spawn or connection fails.
    pub fn spawn_and_connect(&mut self) -> Result<(), PluginError> {
        let entry_point = self.find_entry_point()?;
        let runtime = self.find_runtime();

        tracing::info!(
            runtime = %runtime,
            entry = %entry_point.display(),
            address = %self.ipc_address,
            "Spawning TypeScript plugin host"
        );

        let child = std::process::Command::new(&runtime)
            .arg(&entry_point)
            .env("OPENCLAW_IPC_ADDRESS", &self.ipc_address)
            .env("OPENCLAW_PLUGINS_DIR", &self.plugins_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| PluginError::LoadFailed(format!("Failed to spawn plugin host: {e}")))?;

        self.child_process = Some(child);

        // Wait briefly for the process to start its IPC server
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Connect as client
        self.connect(&self.ipc_address.clone())?;

        // Load skill manifest
        self.manifest = self.load_skills().ok();

        Ok(())
    }

    /// Find the plugin host entry point.
    fn find_entry_point(&self) -> Result<PathBuf, PluginError> {
        let candidates = [
            "plugin-host.js",
            "plugin-host.ts",
            "index.js",
            "index.ts",
            "host.js",
            "host.ts",
        ];

        for name in &candidates {
            let path = self.plugins_dir.join(name);
            if path.exists() {
                return Ok(path);
            }
        }

        Err(PluginError::LoadFailed(format!(
            "No plugin host entry point found in {}",
            self.plugins_dir.display()
        )))
    }

    /// Find the best JavaScript runtime (bun > node).
    fn find_runtime(&self) -> String {
        if which_exists("bun") {
            "bun".to_string()
        } else {
            "node".to_string()
        }
    }

    /// Check if the host process is still running.
    #[must_use]
    pub fn is_running(&mut self) -> bool {
        match &mut self.child_process {
            Some(child) => child.try_wait().ok().flatten().is_none(),
            None => self.transport.is_some(),
        }
    }

    /// Stop the plugin host process.
    pub fn stop(&mut self) {
        if let Some(mut child) = self.child_process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.transport = None;
        self.manifest = None;
    }

    /// Get the cached skill manifest.
    #[must_use]
    pub fn skill_manifest(&self) -> Option<&SkillManifest> {
        self.manifest.as_ref()
    }

    /// Load skills from TypeScript layer.
    ///
    /// # Errors
    ///
    /// Returns error if not connected or IPC fails.
    pub fn load_skills(&self) -> Result<SkillManifest, PluginError> {
        let transport = self
            .transport
            .as_ref()
            .ok_or_else(|| PluginError::Ipc("Not connected".to_string()))?;

        let request = IpcMessage::request("loadSkills", serde_json::json!({}));
        let response = transport
            .request(&request)
            .map_err(|e| PluginError::Ipc(e.to_string()))?;

        if let openclaw_ipc::messages::IpcPayload::Response(resp) = response.payload {
            if resp.success {
                let manifest: SkillManifest =
                    serde_json::from_value(resp.result.unwrap_or_default())
                        .map_err(|e| PluginError::Ipc(e.to_string()))?;
                return Ok(manifest);
            } else {
                return Err(PluginError::Ipc(resp.error.unwrap_or_default()));
            }
        }

        Err(PluginError::Ipc("Invalid response".to_string()))
    }

    /// Execute a tool registered by TypeScript plugin.
    ///
    /// # Errors
    ///
    /// Returns error if not connected or tool execution fails.
    pub fn execute_tool(
        &self,
        name: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, PluginError> {
        let transport = self
            .transport
            .as_ref()
            .ok_or_else(|| PluginError::Ipc("Not connected".to_string()))?;

        let request = IpcMessage::request(
            "executeTool",
            serde_json::json!({
                "name": name,
                "params": params
            }),
        );

        let response = transport
            .request(&request)
            .map_err(|e| PluginError::Ipc(e.to_string()))?;

        if let openclaw_ipc::messages::IpcPayload::Response(resp) = response.payload {
            if resp.success {
                return Ok(resp.result.unwrap_or_default());
            } else {
                return Err(PluginError::Ipc(resp.error.unwrap_or_default()));
            }
        }

        Err(PluginError::Ipc("Invalid response".to_string()))
    }

    /// Call a plugin hook.
    ///
    /// # Errors
    ///
    /// Returns error if not connected or hook execution fails.
    pub fn call_hook(
        &self,
        hook: &str,
        data: serde_json::Value,
    ) -> Result<serde_json::Value, PluginError> {
        let transport = self
            .transport
            .as_ref()
            .ok_or_else(|| PluginError::Ipc("Not connected".to_string()))?;

        let request = IpcMessage::request(
            "callHook",
            serde_json::json!({
                "hook": hook,
                "data": data
            }),
        );

        let response = transport
            .request(&request)
            .map_err(|e| PluginError::Ipc(e.to_string()))?;

        if let openclaw_ipc::messages::IpcPayload::Response(resp) = response.payload {
            if resp.success {
                return Ok(resp.result.unwrap_or_default());
            } else {
                return Err(PluginError::Ipc(resp.error.unwrap_or_default()));
            }
        }

        Err(PluginError::Ipc("Invalid response".to_string()))
    }
}

impl Drop for TsPluginBridge {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Implement the Plugin trait so the bridge can be registered.
#[async_trait]
impl Plugin for TsPluginBridge {
    fn id(&self) -> &str {
        "ts-bridge"
    }

    fn name(&self) -> &str {
        "TypeScript Plugin Bridge"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn hooks(&self) -> &[PluginHook] {
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
        let hook_name = match hook {
            PluginHook::BeforeMessage => "beforeMessage",
            PluginHook::AfterMessage => "afterMessage",
            PluginHook::BeforeToolCall => "beforeToolCall",
            PluginHook::AfterToolCall => "afterToolCall",
            PluginHook::SessionStart => "sessionStart",
            PluginHook::SessionEnd => "sessionEnd",
            PluginHook::AgentResponse => "agentResponse",
            PluginHook::Error => "error",
        };

        self.call_hook(hook_name, data)
    }

    async fn activate(&self) -> Result<(), PluginError> {
        // Already handled by spawn_and_connect or connect
        Ok(())
    }

    async fn deactivate(&self) -> Result<(), PluginError> {
        // Handled by Drop
        Ok(())
    }
}

/// Skill manifest from TypeScript layer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillManifest {
    /// Available skills.
    pub skills: Vec<SkillEntry>,
    /// Formatted prompt for LLM.
    pub prompt: String,
}

/// Skill entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillEntry {
    /// Skill name.
    pub name: String,
    /// Skill description.
    pub description: String,
    /// Slash command (e.g., "/commit").
    pub slash_command: Option<String>,
}

/// Discover TypeScript plugins in a directory.
///
/// Scans for directories containing `package.json` with an `openclaw` plugin entry.
pub fn discover_plugins(plugins_dir: &Path) -> Vec<PluginInfo> {
    let mut plugins = Vec::new();

    let entries = match std::fs::read_dir(plugins_dir) {
        Ok(entries) => entries,
        Err(_) => return plugins,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let pkg_json = path.join("package.json");
        if !pkg_json.exists() {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(&pkg_json) {
            if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                // Check for openclaw plugin marker
                if pkg.get("openclaw").is_some() || pkg.get("openclaw-plugin").is_some() {
                    let name = pkg["name"]
                        .as_str()
                        .unwrap_or("unknown")
                        .to_string();
                    let version = pkg["version"]
                        .as_str()
                        .unwrap_or("0.0.0")
                        .to_string();
                    let description = pkg["description"]
                        .as_str()
                        .unwrap_or("")
                        .to_string();

                    plugins.push(PluginInfo {
                        name,
                        version,
                        description,
                        path: path.clone(),
                    });
                }
            }
        }
    }

    plugins
}

/// Information about a discovered plugin.
#[derive(Debug, Clone)]
pub struct PluginInfo {
    /// Plugin name (from package.json).
    pub name: String,
    /// Plugin version.
    pub version: String,
    /// Plugin description.
    pub description: String,
    /// Path to plugin directory.
    pub path: PathBuf,
}

/// Check if a command exists on PATH.
fn which_exists(cmd: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths).any(|dir| {
                dir.join(cmd).exists()
                    || dir.join(format!("{cmd}.exe")).exists()
            })
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_bridge_creation() {
        let dir = tempdir().unwrap();
        let bridge = TsPluginBridge::new(dir.path());
        assert_eq!(bridge.id(), "ts-bridge");
    }

    #[test]
    fn test_discover_no_plugins() {
        let dir = tempdir().unwrap();
        let plugins = discover_plugins(dir.path());
        assert!(plugins.is_empty());
    }

    #[test]
    fn test_discover_with_plugin() {
        let dir = tempdir().unwrap();
        let plugin_dir = dir.path().join("test-plugin");
        std::fs::create_dir(&plugin_dir).unwrap();
        std::fs::write(
            plugin_dir.join("package.json"),
            r#"{"name": "test-plugin", "version": "1.0.0", "openclaw": {}}"#,
        )
        .unwrap();

        let plugins = discover_plugins(dir.path());
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "test-plugin");
    }
}
