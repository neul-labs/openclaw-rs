//! Onboarding wizard command.

use crate::ui::{
    self,
    prompts::{self, AuthProvider, OnboardFlow},
};
use anyhow::Result;
use openclaw_core::{ApiKey, Config, CredentialStore};
use std::path::PathBuf;

/// Onboard command arguments.
#[derive(Debug, Clone)]
pub struct OnboardArgs {
    /// Non-interactive mode.
    pub non_interactive: bool,
    /// Accept risk acknowledgement.
    pub accept_risk: bool,
    /// Flow selection.
    pub flow: Option<String>,
    /// Auth provider choice.
    pub auth_choice: Option<String>,
    /// API key for selected provider.
    pub api_key: Option<String>,
    /// Install daemon after setup.
    pub install_daemon: bool,
}

impl Default for OnboardArgs {
    fn default() -> Self {
        Self {
            non_interactive: false,
            accept_risk: false,
            flow: None,
            auth_choice: None,
            api_key: None,
            install_daemon: false,
        }
    }
}

/// Run the onboarding wizard.
pub async fn run_onboard(args: OnboardArgs) -> Result<()> {
    ui::banner();
    ui::header("Welcome to OpenClaw Setup");

    // Step 1: Risk acknowledgement
    if !args.accept_risk {
        if args.non_interactive {
            anyhow::bail!("Non-interactive mode requires --accept-risk flag");
        }

        let accepted = prompts::risk_acknowledgement()?;
        if !accepted {
            ui::error("Setup cancelled. You must accept the risks to continue.");
            return Ok(());
        }
    }

    ui::success("Risk acknowledgement accepted");

    // Step 2: Check for existing config
    let config_path = get_config_path();
    let existing_config = Config::load_default().ok();

    if existing_config.is_some() {
        ui::info("Existing configuration found");

        if !args.non_interactive {
            let options = [
                ("Keep", "Keep existing values, update only new settings"),
                ("Reset", "Start fresh with new configuration"),
            ];

            let choice = prompts::select_with_help("Existing config found", &options)?;

            if choice == 1 {
                ui::warning("Resetting configuration...");
                // Will create new config below
            }
        }
    }

    // Step 3: Select flow
    let flow = if let Some(f) = &args.flow {
        match f.to_lowercase().as_str() {
            "quickstart" | "quick" => OnboardFlow::QuickStart,
            "advanced" | "manual" => OnboardFlow::Advanced,
            _ => {
                if args.non_interactive {
                    OnboardFlow::QuickStart
                } else {
                    prompts::select_onboard_flow()?
                }
            }
        }
    } else if args.non_interactive {
        OnboardFlow::QuickStart
    } else {
        prompts::select_onboard_flow()?
    };

    ui::info(&format!("Using {} mode", flow));

    // Step 4: Gateway configuration
    ui::header("Gateway Configuration");

    let (bind_address, port) = match flow {
        OnboardFlow::QuickStart => {
            ui::info("Gateway: localhost:18789 (loopback only)");
            ("127.0.0.1".to_string(), 18789u16)
        }
        OnboardFlow::Advanced => {
            if args.non_interactive {
                ("127.0.0.1".to_string(), 18789u16)
            } else {
                let port_str = prompts::input_with_default("Gateway port", "18789")?;
                let port: u16 = port_str.parse().unwrap_or(18789);

                let bind_options = [
                    ("Loopback", "127.0.0.1 - local access only (recommended)"),
                    ("LAN", "0.0.0.0 - accessible from local network"),
                ];
                let bind_choice = prompts::select_with_help("Bind address", &bind_options)?;

                let bind = match bind_choice {
                    0 => "127.0.0.1".to_string(),
                    _ => "0.0.0.0".to_string(),
                };

                (bind, port)
            }
        }
    };

    ui::success(&format!("Gateway configured: {}:{}", bind_address, port));

    // Step 5: Authentication setup
    ui::header("Authentication Setup");

    let provider = if let Some(auth) = &args.auth_choice {
        match auth.to_lowercase().as_str() {
            "anthropic" => AuthProvider::Anthropic,
            "openai" => AuthProvider::OpenAI,
            "openrouter" => AuthProvider::OpenRouter,
            "skip" => AuthProvider::Skip,
            _ => {
                if args.non_interactive {
                    AuthProvider::Skip
                } else {
                    prompts::select_auth_provider()?
                }
            }
        }
    } else if args.non_interactive {
        AuthProvider::Skip
    } else {
        prompts::select_auth_provider()?
    };

    if provider != AuthProvider::Skip {
        let api_key = if let Some(key) = &args.api_key {
            key.clone()
        } else if args.non_interactive {
            anyhow::bail!("API key required in non-interactive mode");
        } else {
            prompts::password(&format!("Enter {} API key", provider))?
        };

        // Store the credential
        let cred_path = get_credentials_path();
        std::fs::create_dir_all(&cred_path)?;

        // Generate encryption key (in production, derive from master password)
        let encryption_key: [u8; 32] = rand::random();
        let store = CredentialStore::new(encryption_key, cred_path);

        let provider_name = match provider {
            AuthProvider::Anthropic => "anthropic",
            AuthProvider::OpenAI => "openai",
            AuthProvider::OpenRouter => "openrouter",
            AuthProvider::Skip => unreachable!(),
        };

        store.store(provider_name, &ApiKey::new(api_key))?;
        ui::success(&format!("{} credentials stored", provider));
    } else {
        ui::info("Skipping authentication setup");
    }

    // Step 6: Workspace setup
    ui::header("Workspace Setup");

    let workspace = get_workspace_path();
    std::fs::create_dir_all(&workspace)?;
    ui::success(&format!("Workspace: {}", workspace.display()));

    // Step 7: Write configuration
    ui::header("Saving Configuration");

    let config = create_config(&bind_address, port, &workspace, provider);
    let config_json = serde_json::to_string_pretty(&config)?;

    std::fs::create_dir_all(config_path.parent().unwrap())?;
    std::fs::write(&config_path, config_json)?;

    ui::success(&format!("Configuration saved: {}", config_path.display()));

    // Step 8: Shell completion setup
    ui::header("Shell Completion");

    if !args.non_interactive {
        let install_completion = prompts::confirm("Install shell completions?")?;
        if install_completion {
            ui::info("Run 'openclaw completion --install' to set up completions");
        }
    }

    // Step 9: Daemon installation
    if args.install_daemon {
        ui::header("Daemon Installation");
        ui::info("Run 'openclaw daemon install' to install as a system service");
    }

    // Step 10: Summary
    ui::header("Setup Complete!");

    println!();
    ui::kv("Config", &config_path.display().to_string());
    ui::kv("Workspace", &workspace.display().to_string());
    ui::kv("Gateway", &format!("{}:{}", bind_address, port));

    println!();
    ui::info("Next steps:");
    println!("  1. Start the gateway: openclaw gateway run");
    println!("  2. Check status: openclaw status");
    println!("  3. Run diagnostics: openclaw doctor");

    Ok(())
}

/// Get the config file path.
fn get_config_path() -> PathBuf {
    if let Ok(path) = std::env::var("OPENCLAW_CONFIG_PATH") {
        return PathBuf::from(path);
    }

    if let Ok(state_dir) = std::env::var("OPENCLAW_STATE_DIR") {
        return PathBuf::from(state_dir).join("openclaw.json");
    }

    dirs::home_dir()
        .map(|h| h.join(".openclaw").join("openclaw.json"))
        .unwrap_or_else(|| PathBuf::from(".openclaw/openclaw.json"))
}

/// Get the credentials directory path.
fn get_credentials_path() -> PathBuf {
    if let Ok(path) = std::env::var("OPENCLAW_OAUTH_DIR") {
        return PathBuf::from(path);
    }

    dirs::home_dir()
        .map(|h| h.join(".openclaw").join("credentials"))
        .unwrap_or_else(|| PathBuf::from(".openclaw/credentials"))
}

/// Get the workspace path.
fn get_workspace_path() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join(".openclaw").join("workspace"))
        .unwrap_or_else(|| PathBuf::from(".openclaw/workspace"))
}

/// Create a config struct from onboard settings.
fn create_config(
    bind_address: &str,
    port: u16,
    workspace: &PathBuf,
    provider: AuthProvider,
) -> serde_json::Value {
    let default_model = match provider {
        AuthProvider::Anthropic => "claude-sonnet-4-20250514",
        AuthProvider::OpenAI => "gpt-4o",
        AuthProvider::OpenRouter => "anthropic/claude-sonnet-4-20250514",
        AuthProvider::Skip => "claude-sonnet-4-20250514",
    };

    let default_provider = match provider {
        AuthProvider::Anthropic => "anthropic",
        AuthProvider::OpenAI => "openai",
        AuthProvider::OpenRouter => "openrouter",
        AuthProvider::Skip => "anthropic",
    };

    serde_json::json!({
        "gateway": {
            "mode": "local",
            "port": port,
            "bind": if bind_address == "127.0.0.1" { "loopback" } else { "lan" },
            "auth": {
                "mode": "token",
                "token": generate_token()
            }
        },
        "agents": {
            "defaults": {
                "workspace": workspace.display().to_string(),
                "model": default_model,
                "provider": default_provider
            }
        },
        "channels": {},
        "wizard": {
            "lastRunAt": chrono::Utc::now().to_rfc3339(),
            "lastRunVersion": env!("CARGO_PKG_VERSION"),
            "lastRunCommand": "onboard"
        }
    })
}

/// Generate a random token for gateway auth.
fn generate_token() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
