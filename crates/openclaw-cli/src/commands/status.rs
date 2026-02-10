//! Status command - show gateway, agents, and channels status.

use crate::ui::{self, HealthStatus};
use anyhow::Result;
use std::time::Duration;

/// Status command arguments.
#[derive(Debug, Clone, Default)]
pub struct StatusArgs {
    /// Show all details.
    pub all: bool,
    /// Probe services for connectivity.
    pub deep: bool,
}

/// Run the status command.
pub async fn run_status(args: StatusArgs) -> Result<()> {
    ui::header("OpenClaw Status");

    // Gateway status
    println!();
    ui::info("Gateway");
    let gateway_status = check_gateway_status(args.deep).await;

    match gateway_status {
        GatewayStatus::Running { port, version } => {
            ui::health_check("Status", HealthStatus::Ok, Some("running"));
            ui::kv("  Port", &port.to_string());
            if let Some(v) = version {
                ui::kv("  Version", &v);
            }
        }
        GatewayStatus::NotRunning => {
            ui::health_check("Status", HealthStatus::Warning, Some("not running"));
            ui::info("  Start with: openclaw gateway run");
        }
        GatewayStatus::Error(msg) => {
            ui::health_check("Status", HealthStatus::Error, Some(&msg));
        }
    }

    // Config status
    println!();
    ui::info("Configuration");
    if let Ok(config) = openclaw_core::Config::load_default() {
        ui::health_check("Config", HealthStatus::Ok, Some("loaded"));
        if args.all {
            ui::kv("  Gateway Port", &config.gateway.port.to_string());
            // Show default agent model if configured
            if let Some(default_agent) = config.agents.get("default") {
                ui::kv("  Default Model", &default_agent.model);
            }
        }
    } else {
        ui::health_check("Config", HealthStatus::Warning, Some("not found"));
        ui::info("  Run 'openclaw onboard' to configure");
    }

    // Sandbox status
    println!();
    ui::info("Sandbox");
    if openclaw_agents::sandbox::is_sandbox_available() {
        let sandbox_type = if cfg!(target_os = "linux") {
            "bubblewrap"
        } else if cfg!(target_os = "macos") {
            "sandbox-exec"
        } else if cfg!(target_os = "windows") {
            "Job Objects"
        } else {
            "unknown"
        };
        ui::health_check("Sandbox", HealthStatus::Ok, Some(sandbox_type));
    } else {
        ui::health_check("Sandbox", HealthStatus::Warning, Some("not available"));
    }

    // Credentials status
    println!();
    ui::info("Credentials");
    let cred_path = dirs::home_dir()
        .map(|h| h.join(".openclaw").join("credentials"))
        .unwrap_or_default();

    if cred_path.exists() {
        let count = std::fs::read_dir(&cred_path)
            .map(|entries| entries.filter_map(std::result::Result::ok).count())
            .unwrap_or(0);

        if count > 0 {
            ui::health_check(
                "Credentials",
                HealthStatus::Ok,
                Some(&format!("{count} provider(s)")),
            );

            if args.all {
                // List providers
                if let Ok(entries) = std::fs::read_dir(&cred_path) {
                    for entry in entries.filter_map(std::result::Result::ok) {
                        if let Some(name) = entry.file_name().to_str() {
                            if let Some(provider) = name.strip_suffix(".enc") {
                                ui::kv("  ", provider);
                            }
                        }
                    }
                }
            }
        } else {
            ui::health_check("Credentials", HealthStatus::Warning, Some("no API keys"));
        }
    } else {
        ui::health_check("Credentials", HealthStatus::Warning, Some("not configured"));
    }

    // Deep probe
    if args.deep {
        println!();
        ui::info("Deep Probe");

        // Probe gateway health endpoint
        ui::info("  Probing gateway...");
        match probe_gateway_health().await {
            Ok(health) => {
                ui::health_check("  Health endpoint", HealthStatus::Ok, Some(&health));
            }
            Err(e) => {
                ui::health_check("  Health endpoint", HealthStatus::Error, Some(&e));
            }
        }
    }

    Ok(())
}

/// Gateway status result.
enum GatewayStatus {
    Running { port: u16, version: Option<String> },
    NotRunning,
    Error(String),
}

/// Check gateway status.
async fn check_gateway_status(deep: bool) -> GatewayStatus {
    let port = get_gateway_port();

    // Try to connect
    match tokio::net::TcpStream::connect(format!("127.0.0.1:{port}")).await {
        Ok(_) => {
            if deep {
                // Try to get version from health endpoint
                match probe_gateway_version(port).await {
                    Ok(version) => GatewayStatus::Running {
                        port,
                        version: Some(version),
                    },
                    Err(_) => GatewayStatus::Running {
                        port,
                        version: None,
                    },
                }
            } else {
                GatewayStatus::Running {
                    port,
                    version: None,
                }
            }
        }
        Err(_) => GatewayStatus::NotRunning,
    }
}

/// Get configured gateway port.
fn get_gateway_port() -> u16 {
    // Try environment variable
    if let Ok(port) = std::env::var("OPENCLAW_GATEWAY_PORT") {
        if let Ok(p) = port.parse() {
            return p;
        }
    }

    // Try config
    if let Ok(config) = openclaw_core::Config::load_default() {
        return config.gateway.port;
    }

    // Default
    18789
}

/// Probe gateway version.
async fn probe_gateway_version(port: u16) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(format!("http://127.0.0.1:{port}/health"))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status().is_success() {
        let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        if let Some(version) = body.get("version").and_then(|v| v.as_str()) {
            return Ok(version.to_string());
        }
    }

    Err("Unknown version".to_string())
}

/// Probe gateway health endpoint.
async fn probe_gateway_health() -> Result<String, String> {
    let port = get_gateway_port();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(format!("http://127.0.0.1:{port}/health"))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status().is_success() {
        Ok(format!("HTTP {}", resp.status()))
    } else {
        Err(format!("HTTP {}", resp.status()))
    }
}
