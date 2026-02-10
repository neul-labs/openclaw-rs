//! Gateway command - start and manage the gateway server.

use crate::ui;
use anyhow::Result;
use openclaw_core::config::BindMode;

/// Gateway command arguments.
#[derive(Debug, Clone)]
pub struct GatewayArgs {
    /// Subcommand.
    pub action: GatewayAction,
}

/// Gateway actions.
#[derive(Debug, Clone)]
pub enum GatewayAction {
    Run {
        /// Port to listen on.
        port: Option<u16>,
        /// Bind address.
        bind: Option<String>,
        /// Force start even if another instance is running.
        force: bool,
    },
    Status,
}

impl Default for GatewayArgs {
    fn default() -> Self {
        Self {
            action: GatewayAction::Status,
        }
    }
}

/// Run the gateway command.
pub async fn run_gateway(args: GatewayArgs) -> Result<()> {
    match args.action {
        GatewayAction::Run { port, bind, force } => run_gateway_server(port, bind, force).await,
        GatewayAction::Status => gateway_status().await,
    }
}

/// Start the gateway server.
async fn run_gateway_server(port: Option<u16>, bind: Option<String>, force: bool) -> Result<()> {
    // Load configuration
    let config = if let Ok(c) = openclaw_core::Config::load_default() {
        c
    } else {
        ui::warning("No configuration found, using defaults");
        ui::info("Run 'openclaw onboard' for full setup");
        openclaw_core::Config::default()
    };

    // Determine port and bind address
    let server_port = port.unwrap_or(config.gateway.port);
    let bind_address = bind.unwrap_or_else(|| match &config.gateway.mode {
        BindMode::Local => "127.0.0.1".to_string(),
        BindMode::Public => "0.0.0.0".to_string(),
        BindMode::Custom(addr) => addr.clone(),
    });

    // Check if port is already in use
    if !force {
        if let Ok(listener) = std::net::TcpListener::bind(format!("{bind_address}:{server_port}")) {
            drop(listener);
        } else {
            ui::error(&format!(
                "Port {server_port} is already in use. Use --force to override."
            ));
            return Ok(());
        }
    }

    ui::header("Starting OpenClaw Gateway");
    ui::kv("Address", &format!("{bind_address}:{server_port}"));
    let mode_str = match &config.gateway.mode {
        BindMode::Local => "local",
        BindMode::Public => "public",
        BindMode::Custom(_) => "custom",
    };
    ui::kv("Mode", mode_str);
    println!();

    let gateway_config = openclaw_gateway::GatewayConfig {
        port: server_port,
        bind_address,
        cors: true,
        ..Default::default()
    };

    ui::info("Gateway is starting...");
    ui::info("Press Ctrl+C to stop");
    println!();

    // Start the gateway
    openclaw_gateway::start(gateway_config).await?;

    Ok(())
}

/// Check gateway status.
async fn gateway_status() -> Result<()> {
    ui::header("Gateway Status");

    // Get configured port
    let port = match openclaw_core::Config::load_default() {
        Ok(c) => c.gateway.port,
        Err(_) => 18789,
    };

    // Try to connect
    if let Ok(_) = tokio::net::TcpStream::connect(format!("127.0.0.1:{port}")).await {
        ui::success(&format!("Gateway is running on port {port}"));

        // Try health check
        let client = reqwest::Client::new();
        match client
            .get(format!("http://127.0.0.1:{port}/health"))
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
        {
            Ok(resp) => {
                if resp.status().is_success() {
                    if let Ok(body) = resp.json::<serde_json::Value>().await {
                        if let Some(version) = body.get("version").and_then(|v| v.as_str()) {
                            ui::kv("Version", version);
                        }
                        if let Some(status) = body.get("status").and_then(|v| v.as_str()) {
                            ui::kv("Status", status);
                        }
                    }
                }
            }
            Err(_) => {
                ui::warning("Health check failed");
            }
        }
    } else {
        ui::warning(&format!("Gateway is not running on port {port}"));
        ui::info("Start with: openclaw gateway run");
    }

    Ok(())
}
