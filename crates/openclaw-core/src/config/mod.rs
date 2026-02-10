//! Configuration loading and validation.
//!
//! Supports JSON5 format for compatibility with existing `OpenClaw` config.
//! Config location: `~/.openclaw/openclaw.json`

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Configuration errors.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// IO error reading config file.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON5 parsing error.
    #[error("Parse error: {0}")]
    Parse(#[from] json5::Error),

    /// Config validation error.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Missing required field.
    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// Main configuration structure.
///
/// Matches the existing `OpenClaw` JSON5 config schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct Config {
    /// Gateway configuration.
    #[serde(default)]
    pub gateway: GatewayConfig,

    /// Agent configurations by ID.
    #[serde(default)]
    pub agents: HashMap<String, AgentConfig>,

    /// Channel configurations.
    #[serde(default)]
    pub channels: ChannelsConfig,

    /// Provider configurations.
    #[serde(default)]
    pub providers: ProvidersConfig,

    /// Global settings.
    #[serde(default)]
    pub settings: GlobalSettings,
}

impl Config {
    /// Load configuration from the default location.
    ///
    /// # Errors
    ///
    /// Returns error if config cannot be loaded or parsed.
    pub fn load_default() -> Result<Self, ConfigError> {
        let path = Self::default_path();
        if path.exists() {
            Self::load(&path)
        } else {
            Ok(Self::default())
        }
    }

    /// Load configuration from a specific path.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or parsed.
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = json5::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Save configuration to a path.
    ///
    /// # Errors
    ///
    /// Returns error if serialization or file write fails.
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| ConfigError::Validation(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get the default config file path.
    #[must_use]
    pub fn default_path() -> PathBuf {
        Self::state_dir().join("openclaw.json")
    }

    /// Get the `OpenClaw` state directory.
    ///
    /// Uses `OPENCLAW_STATE_DIR` env var if set, otherwise `~/.openclaw`.
    #[must_use]
    pub fn state_dir() -> PathBuf {
        if let Ok(dir) = std::env::var("OPENCLAW_STATE_DIR") {
            PathBuf::from(dir)
        } else if let Some(home) = dirs::home_dir() {
            home.join(".openclaw")
        } else {
            PathBuf::from(".openclaw")
        }
    }

    /// Get the credentials directory.
    #[must_use]
    pub fn credentials_dir() -> PathBuf {
        Self::state_dir().join("credentials")
    }

    /// Get the sessions directory.
    #[must_use]
    pub fn sessions_dir() -> PathBuf {
        Self::state_dir().join("sessions")
    }

    /// Get the agents directory.
    #[must_use]
    pub fn agents_dir() -> PathBuf {
        Self::state_dir().join("agents")
    }

    /// Validate the configuration.
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate gateway port
        if self.gateway.port == 0 {
            return Err(ConfigError::Validation(
                "Gateway port cannot be 0".to_string(),
            ));
        }

        // Validate agent configs
        for (id, agent) in &self.agents {
            if agent.model.is_empty() {
                return Err(ConfigError::Validation(format!(
                    "Agent '{id}' has empty model"
                )));
            }
        }

        Ok(())
    }

    /// Get agent config by ID, falling back to default.
    #[must_use]
    pub fn get_agent(&self, id: &str) -> AgentConfig {
        self.agents
            .get(id)
            .cloned()
            .unwrap_or_else(AgentConfig::default)
    }
}

/// Gateway server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayConfig {
    /// Port to listen on.
    #[serde(default = "default_port")]
    pub port: u16,

    /// Bind address mode.
    #[serde(default)]
    pub mode: BindMode,

    /// Enable CORS.
    #[serde(default = "default_true")]
    pub cors: bool,

    /// Request timeout in seconds.
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            mode: BindMode::default(),
            cors: true,
            timeout_secs: default_timeout(),
        }
    }
}

const fn default_port() -> u16 {
    18789
}

const fn default_timeout() -> u64 {
    300
}

const fn default_true() -> bool {
    true
}

/// Gateway bind mode.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BindMode {
    /// Bind to localhost only.
    #[default]
    Local,
    /// Bind to all interfaces.
    Public,
    /// Custom bind address.
    Custom(String),
}

/// Agent configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    /// Model to use (e.g., "claude-3-5-sonnet-20241022").
    #[serde(default = "default_model")]
    pub model: String,

    /// Provider to use.
    #[serde(default = "default_provider")]
    pub provider: String,

    /// System prompt.
    #[serde(default)]
    pub system_prompt: Option<String>,

    /// Maximum tokens in response.
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// Temperature for sampling.
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Enabled tools.
    #[serde(default)]
    pub tools: Vec<String>,

    /// Allowlist patterns for this agent.
    #[serde(default)]
    pub allowlist: Vec<AllowlistEntry>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model: default_model(),
            provider: default_provider(),
            system_prompt: None,
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
            tools: vec![],
            allowlist: vec![],
        }
    }
}

fn default_model() -> String {
    "claude-3-5-sonnet-20241022".to_string()
}

fn default_provider() -> String {
    "anthropic".to_string()
}

const fn default_max_tokens() -> u32 {
    4096
}

const fn default_temperature() -> f32 {
    0.7
}

/// Allowlist entry for agent access control.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowlistEntry {
    /// Channel pattern (e.g., "telegram", "*").
    pub channel: String,

    /// Peer ID pattern (e.g., "123456789", "*").
    pub peer_id: String,

    /// Optional label for this entry.
    #[serde(default)]
    pub label: Option<String>,
}

/// Channel configurations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelsConfig {
    /// Telegram channel config.
    #[serde(default)]
    pub telegram: Option<TelegramConfig>,

    /// Discord channel config.
    #[serde(default)]
    pub discord: Option<DiscordConfig>,

    /// Slack channel config.
    #[serde(default)]
    pub slack: Option<SlackConfig>,

    /// Signal channel config.
    #[serde(default)]
    pub signal: Option<SignalConfig>,

    /// Matrix channel config.
    #[serde(default)]
    pub matrix: Option<MatrixConfig>,
}

/// Telegram channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegramConfig {
    /// Bot token.
    pub bot_token: Option<String>,

    /// Enable webhook mode.
    #[serde(default)]
    pub webhook: bool,

    /// Webhook URL (if webhook mode).
    #[serde(default)]
    pub webhook_url: Option<String>,
}

/// Discord channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordConfig {
    /// Bot token.
    pub bot_token: Option<String>,

    /// Application ID.
    pub application_id: Option<String>,
}

/// Slack channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlackConfig {
    /// Bot token.
    pub bot_token: Option<String>,

    /// App token (for socket mode).
    pub app_token: Option<String>,
}

/// Signal channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalConfig {
    /// Phone number.
    pub phone_number: Option<String>,
}

/// Matrix channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatrixConfig {
    /// Homeserver URL.
    pub homeserver: Option<String>,

    /// User ID.
    pub user_id: Option<String>,

    /// Access token.
    pub access_token: Option<String>,
}

/// Provider configurations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvidersConfig {
    /// Anthropic configuration.
    #[serde(default)]
    pub anthropic: Option<AnthropicConfig>,

    /// `OpenAI` configuration.
    #[serde(default)]
    pub openai: Option<OpenAIConfig>,

    /// Ollama configuration.
    #[serde(default)]
    pub ollama: Option<OllamaConfig>,
}

/// Anthropic provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnthropicConfig {
    /// API key (prefer credential store).
    pub api_key: Option<String>,

    /// Base URL override.
    #[serde(default)]
    pub base_url: Option<String>,
}

/// `OpenAI` provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAIConfig {
    /// API key (prefer credential store).
    pub api_key: Option<String>,

    /// Base URL override.
    #[serde(default)]
    pub base_url: Option<String>,

    /// Organization ID.
    #[serde(default)]
    pub org_id: Option<String>,
}

/// Ollama provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaConfig {
    /// Base URL.
    #[serde(default = "default_ollama_url")]
    pub base_url: String,
}

fn default_ollama_url() -> String {
    "http://localhost:11434".to_string()
}

/// Global settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct GlobalSettings {
    /// Enable debug logging.
    #[serde(default)]
    pub debug: bool,

    /// Log format.
    #[serde(default)]
    pub log_format: LogFormat,

    /// Telemetry enabled.
    #[serde(default)]
    pub telemetry: bool,
}

/// Log format.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// Human-readable format.
    #[default]
    Pretty,
    /// JSON format.
    Json,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.gateway.port, 18789);
    }

    #[test]
    fn test_config_roundtrip() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("config.json");

        let mut config = Config::default();
        config.agents.insert(
            "test".to_string(),
            AgentConfig {
                model: "gpt-4".to_string(),
                ..Default::default()
            },
        );

        config.save(&path).unwrap();

        let loaded = Config::load(&path).unwrap();
        assert_eq!(loaded.agents.get("test").unwrap().model, "gpt-4");
    }

    #[test]
    fn test_json5_parsing() {
        let json5_content = r#"{
            // This is a comment
            gateway: {
                port: 8080,
            },
            agents: {
                default: {
                    model: "claude-3-5-sonnet-20241022",
                    // trailing comma
                },
            },
        }"#;

        let config: Config = json5::from_str(json5_content).unwrap();
        assert_eq!(config.gateway.port, 8080);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        config.gateway.port = 0;

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_state_dir() {
        let dir = Config::state_dir();
        assert!(dir.to_str().unwrap().contains("openclaw"));
    }
}
