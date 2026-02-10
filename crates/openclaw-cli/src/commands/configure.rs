//! Configure command - interactive configuration updates.

use crate::ui::{self, prompts};
use anyhow::Result;
use std::path::PathBuf;

/// Configure command arguments.
#[derive(Debug, Clone)]
pub struct ConfigureArgs {
    /// Section to configure.
    pub section: Option<String>,
}

impl Default for ConfigureArgs {
    fn default() -> Self {
        Self { section: None }
    }
}

/// Run the configure command.
pub async fn run_configure(args: ConfigureArgs) -> Result<()> {
    ui::header("OpenClaw Configuration");

    let section = if let Some(s) = args.section {
        s
    } else {
        // Ask which section to configure
        let options = [
            ("auth", "API keys and authentication"),
            ("gateway", "Gateway server settings"),
            ("agents", "Agent defaults and models"),
            ("channels", "Messaging channels"),
            ("workspace", "Workspace directory"),
        ];

        let choice = prompts::select_with_help("What would you like to configure?", &options)?;

        match choice {
            0 => "auth".to_string(),
            1 => "gateway".to_string(),
            2 => "agents".to_string(),
            3 => "channels".to_string(),
            _ => "workspace".to_string(),
        }
    };

    match section.as_str() {
        "auth" => configure_auth().await?,
        "gateway" => configure_gateway().await?,
        "agents" => configure_agents().await?,
        "channels" => configure_channels().await?,
        "workspace" => configure_workspace().await?,
        _ => {
            ui::error(&format!("Unknown section: {}", section));
            ui::info("Valid sections: auth, gateway, agents, channels, workspace");
        }
    }

    Ok(())
}

/// Configure authentication.
async fn configure_auth() -> Result<()> {
    ui::header("Authentication Configuration");

    let options = [
        ("Anthropic", "Claude models"),
        ("OpenAI", "GPT models"),
        ("OpenRouter", "Multiple providers"),
    ];

    let choice = prompts::select_with_help("Select provider to configure", &options)?;

    let provider_name = match choice {
        0 => "anthropic",
        1 => "openai",
        _ => "openrouter",
    };

    let api_key = prompts::password(&format!("Enter {} API key", provider_name))?;

    // Store the credential
    let cred_path = get_credentials_path();
    std::fs::create_dir_all(&cred_path)?;

    // Generate encryption key (in production, derive from master password)
    let encryption_key: [u8; 32] = rand::random();
    let store = openclaw_core::CredentialStore::new(encryption_key, cred_path);

    store.store(provider_name, &openclaw_core::ApiKey::new(api_key))?;
    ui::success(&format!("{} credentials stored", provider_name));

    Ok(())
}

/// Configure gateway settings.
async fn configure_gateway() -> Result<()> {
    ui::header("Gateway Configuration");

    let config_path = get_config_path();
    let mut config = load_config(&config_path)?;

    // Port
    let current_port = config
        .get("gateway")
        .and_then(|g| g.get("port"))
        .and_then(|p| p.as_u64())
        .unwrap_or(18789);

    let port_str = prompts::input_with_default("Gateway port", &current_port.to_string())?;
    let port: u64 = port_str.parse().unwrap_or(18789);

    // Bind address
    let bind_options = [
        ("Loopback", "127.0.0.1 - local access only"),
        ("LAN", "0.0.0.0 - network access"),
    ];
    let bind_choice = prompts::select_with_help("Bind address", &bind_options)?;
    let bind = if bind_choice == 0 { "loopback" } else { "lan" };

    // Update config
    if let Some(gateway) = config.get_mut("gateway") {
        if let Some(obj) = gateway.as_object_mut() {
            obj.insert("port".to_string(), serde_json::json!(port));
            obj.insert("bind".to_string(), serde_json::json!(bind));
        }
    } else {
        config.as_object_mut().unwrap().insert(
            "gateway".to_string(),
            serde_json::json!({
                "mode": "local",
                "port": port,
                "bind": bind
            }),
        );
    }

    save_config(&config_path, &config)?;
    ui::success("Gateway configuration updated");

    Ok(())
}

/// Configure agent defaults.
async fn configure_agents() -> Result<()> {
    ui::header("Agent Configuration");

    let config_path = get_config_path();
    let mut config = load_config(&config_path)?;

    // Model selection
    let models = [
        ("claude-sonnet-4-20250514", "Claude Sonnet 4 (recommended)"),
        ("claude-opus-4-20250514", "Claude Opus 4"),
        ("gpt-4o", "GPT-4o"),
        ("gpt-4-turbo", "GPT-4 Turbo"),
    ];

    let model_choice = prompts::select_with_help("Default model", &models)?;
    let model = models[model_choice].0;

    // Update config
    let agents = config
        .as_object_mut()
        .unwrap()
        .entry("agents")
        .or_insert(serde_json::json!({}));

    let defaults = agents
        .as_object_mut()
        .unwrap()
        .entry("defaults")
        .or_insert(serde_json::json!({}));

    defaults
        .as_object_mut()
        .unwrap()
        .insert("model".to_string(), serde_json::json!(model));

    save_config(&config_path, &config)?;
    ui::success(&format!("Default model set to {}", model));

    Ok(())
}

/// Configure channels.
async fn configure_channels() -> Result<()> {
    ui::header("Channel Configuration");

    let options = [
        ("Telegram", "Telegram bot"),
        ("Discord", "Discord bot"),
        ("Slack", "Slack app"),
        ("Signal", "Signal messenger"),
    ];

    let choice = prompts::select_with_help("Select channel to configure", &options)?;

    match choice {
        0 => {
            ui::info("Telegram Configuration");
            let token = prompts::password("Enter Telegram bot token")?;

            let config_path = get_config_path();
            let mut config = load_config(&config_path)?;

            let channels = config
                .as_object_mut()
                .unwrap()
                .entry("channels")
                .or_insert(serde_json::json!({}));

            channels.as_object_mut().unwrap().insert(
                "telegram".to_string(),
                serde_json::json!({
                    "enabled": true,
                    "botToken": token
                }),
            );

            save_config(&config_path, &config)?;
            ui::success("Telegram configured");
        }
        1 => {
            ui::info("Discord Configuration");
            let token = prompts::password("Enter Discord bot token")?;

            let config_path = get_config_path();
            let mut config = load_config(&config_path)?;

            let channels = config
                .as_object_mut()
                .unwrap()
                .entry("channels")
                .or_insert(serde_json::json!({}));

            channels.as_object_mut().unwrap().insert(
                "discord".to_string(),
                serde_json::json!({
                    "enabled": true,
                    "botToken": token
                }),
            );

            save_config(&config_path, &config)?;
            ui::success("Discord configured");
        }
        2 => {
            ui::info("Slack Configuration");
            ui::warning(
                "Slack requires OAuth setup. Visit https://api.slack.com/apps to create an app.",
            );
        }
        3 => {
            ui::info("Signal Configuration");
            ui::warning("Signal requires signal-cli setup. See documentation for details.");
        }
        _ => {}
    }

    Ok(())
}

/// Configure workspace.
async fn configure_workspace() -> Result<()> {
    ui::header("Workspace Configuration");

    let default_workspace = dirs::home_dir()
        .map(|h| h.join(".openclaw").join("workspace"))
        .unwrap_or_else(|| PathBuf::from(".openclaw/workspace"));

    let workspace_str = prompts::input_with_default(
        "Workspace directory",
        &default_workspace.display().to_string(),
    )?;

    let workspace = PathBuf::from(&workspace_str);
    std::fs::create_dir_all(&workspace)?;

    let config_path = get_config_path();
    let mut config = load_config(&config_path)?;

    let agents = config
        .as_object_mut()
        .unwrap()
        .entry("agents")
        .or_insert(serde_json::json!({}));

    let defaults = agents
        .as_object_mut()
        .unwrap()
        .entry("defaults")
        .or_insert(serde_json::json!({}));

    defaults
        .as_object_mut()
        .unwrap()
        .insert("workspace".to_string(), serde_json::json!(workspace_str));

    save_config(&config_path, &config)?;
    ui::success(&format!("Workspace set to {}", workspace.display()));

    Ok(())
}

/// Get the config file path.
fn get_config_path() -> PathBuf {
    if let Ok(path) = std::env::var("OPENCLAW_CONFIG_PATH") {
        return PathBuf::from(path);
    }

    dirs::home_dir()
        .map(|h| h.join(".openclaw").join("openclaw.json"))
        .unwrap_or_else(|| PathBuf::from(".openclaw/openclaw.json"))
}

/// Get the credentials directory path.
fn get_credentials_path() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join(".openclaw").join("credentials"))
        .unwrap_or_else(|| PathBuf::from(".openclaw/credentials"))
}

/// Load configuration from file.
fn load_config(path: &PathBuf) -> Result<serde_json::Value> {
    if path.exists() {
        let content = std::fs::read_to_string(path)?;
        Ok(json5::from_str(&content)?)
    } else {
        Ok(serde_json::json!({}))
    }
}

/// Save configuration to file.
fn save_config(path: &PathBuf, config: &serde_json::Value) -> Result<()> {
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(path, serde_json::to_string_pretty(config)?)?;
    Ok(())
}
