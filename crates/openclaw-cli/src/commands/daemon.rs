//! Daemon management command.

use crate::ui;
use anyhow::Result;
use std::path::PathBuf;

/// Daemon command arguments.
#[derive(Debug, Clone)]
pub struct DaemonArgs {
    /// Subcommand.
    pub action: DaemonAction,
}

/// Daemon actions.
#[derive(Debug, Clone)]
pub enum DaemonAction {
    Install,
    Uninstall,
    Start,
    Stop,
    Status,
}

impl Default for DaemonArgs {
    fn default() -> Self {
        Self {
            action: DaemonAction::Status,
        }
    }
}

/// Run the daemon command.
pub async fn run_daemon(args: DaemonArgs) -> Result<()> {
    match args.action {
        DaemonAction::Install => install_daemon().await,
        DaemonAction::Uninstall => uninstall_daemon().await,
        DaemonAction::Start => start_daemon().await,
        DaemonAction::Stop => stop_daemon().await,
        DaemonAction::Status => daemon_status().await,
    }
}

/// Install the daemon as a system service.
async fn install_daemon() -> Result<()> {
    ui::header("Installing OpenClaw Daemon");

    #[cfg(target_os = "macos")]
    {
        install_launchd_service()?;
    }

    #[cfg(target_os = "linux")]
    {
        install_systemd_service()?;
    }

    #[cfg(target_os = "windows")]
    {
        ui::warning("Windows service installation not yet implemented");
        ui::info("Run 'openclaw gateway run' manually for now");
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        ui::error("Daemon installation not supported on this platform");
    }

    Ok(())
}

/// Uninstall the daemon.
async fn uninstall_daemon() -> Result<()> {
    ui::header("Uninstalling OpenClaw Daemon");

    #[cfg(target_os = "macos")]
    {
        uninstall_launchd_service()?;
    }

    #[cfg(target_os = "linux")]
    {
        uninstall_systemd_service()?;
    }

    #[cfg(target_os = "windows")]
    {
        ui::warning("Windows service uninstallation not yet implemented");
    }

    Ok(())
}

/// Start the daemon.
async fn start_daemon() -> Result<()> {
    ui::info("Starting OpenClaw daemon...");

    #[cfg(target_os = "macos")]
    {
        let plist_path = get_launchd_plist_path();
        let status = std::process::Command::new("launchctl")
            .args(["load", "-w"])
            .arg(&plist_path)
            .status()?;

        if status.success() {
            ui::success("Daemon started");
        } else {
            ui::error("Failed to start daemon");
        }
    }

    #[cfg(target_os = "linux")]
    {
        let status = std::process::Command::new("systemctl")
            .args(["--user", "start", "openclaw"])
            .status()?;

        if status.success() {
            ui::success("Daemon started");
        } else {
            ui::error("Failed to start daemon");
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        ui::error("Daemon start not supported on this platform");
    }

    Ok(())
}

/// Stop the daemon.
async fn stop_daemon() -> Result<()> {
    ui::info("Stopping OpenClaw daemon...");

    #[cfg(target_os = "macos")]
    {
        let plist_path = get_launchd_plist_path();
        let status = std::process::Command::new("launchctl")
            .args(["unload"])
            .arg(&plist_path)
            .status()?;

        if status.success() {
            ui::success("Daemon stopped");
        } else {
            ui::error("Failed to stop daemon");
        }
    }

    #[cfg(target_os = "linux")]
    {
        let status = std::process::Command::new("systemctl")
            .args(["--user", "stop", "openclaw"])
            .status()?;

        if status.success() {
            ui::success("Daemon stopped");
        } else {
            ui::error("Failed to stop daemon");
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        ui::error("Daemon stop not supported on this platform");
    }

    Ok(())
}

/// Check daemon status.
async fn daemon_status() -> Result<()> {
    ui::header("OpenClaw Daemon Status");

    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("launchctl")
            .args(["list"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("ai.openclaw.gateway") {
            ui::success("Daemon is installed and running");
        } else {
            let plist_path = get_launchd_plist_path();
            if plist_path.exists() {
                ui::warning("Daemon is installed but not running");
                ui::info("Start with: openclaw daemon start");
            } else {
                ui::info("Daemon is not installed");
                ui::info("Install with: openclaw daemon install");
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let output = std::process::Command::new("systemctl")
            .args(["--user", "is-active", "openclaw"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        match stdout.as_str() {
            "active" => {
                ui::success("Daemon is running");
            }
            "inactive" => {
                ui::warning("Daemon is installed but not running");
                ui::info("Start with: openclaw daemon start");
            }
            _ => {
                let service_path = get_systemd_service_path();
                if service_path.exists() {
                    ui::warning("Daemon status unknown");
                } else {
                    ui::info("Daemon is not installed");
                    ui::info("Install with: openclaw daemon install");
                }
            }
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        ui::info("Daemon status not supported on this platform");
    }

    Ok(())
}

// Platform-specific implementations

#[cfg(target_os = "macos")]
fn get_launchd_plist_path() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join("Library/LaunchAgents/ai.openclaw.gateway.plist"))
        .unwrap_or_else(|| PathBuf::from("ai.openclaw.gateway.plist"))
}

#[cfg(target_os = "macos")]
fn install_launchd_service() -> Result<()> {
    let plist_path = get_launchd_plist_path();

    // Find the openclaw binary
    let binary_path = std::env::current_exe()?;

    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>ai.openclaw.gateway</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>gateway</string>
        <string>run</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/openclaw-gateway.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/openclaw-gateway.log</string>
</dict>
</plist>
"#,
        binary_path.display()
    );

    std::fs::create_dir_all(plist_path.parent().unwrap())?;
    std::fs::write(&plist_path, plist_content)?;

    ui::success(&format!(
        "Installed launchd service: {}",
        plist_path.display()
    ));
    ui::info("Start with: openclaw daemon start");

    Ok(())
}

#[cfg(target_os = "macos")]
fn uninstall_launchd_service() -> Result<()> {
    let plist_path = get_launchd_plist_path();

    // Stop first
    let _ = std::process::Command::new("launchctl")
        .args(["unload"])
        .arg(&plist_path)
        .status();

    if plist_path.exists() {
        std::fs::remove_file(&plist_path)?;
        ui::success("Daemon uninstalled");
    } else {
        ui::info("Daemon was not installed");
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn get_systemd_service_path() -> PathBuf {
    dirs::home_dir().map_or_else(
        || PathBuf::from("openclaw.service"),
        |h| h.join(".config/systemd/user/openclaw.service"),
    )
}

#[cfg(target_os = "linux")]
fn install_systemd_service() -> Result<()> {
    let service_path = get_systemd_service_path();

    // Find the openclaw binary
    let binary_path = std::env::current_exe()?;

    let service_content = format!(
        r"[Unit]
Description=OpenClaw Gateway
After=network.target

[Service]
Type=simple
ExecStart={} gateway run
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
",
        binary_path.display()
    );

    std::fs::create_dir_all(service_path.parent().unwrap())?;
    std::fs::write(&service_path, service_content)?;

    // Reload systemd
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();

    // Enable the service
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "enable", "openclaw"])
        .status();

    ui::success(&format!(
        "Installed systemd service: {}",
        service_path.display()
    ));
    ui::info("Start with: openclaw daemon start");

    Ok(())
}

#[cfg(target_os = "linux")]
fn uninstall_systemd_service() -> Result<()> {
    let service_path = get_systemd_service_path();

    // Stop and disable first
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "stop", "openclaw"])
        .status();

    let _ = std::process::Command::new("systemctl")
        .args(["--user", "disable", "openclaw"])
        .status();

    if service_path.exists() {
        std::fs::remove_file(&service_path)?;

        // Reload systemd
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .status();

        ui::success("Daemon uninstalled");
    } else {
        ui::info("Daemon was not installed");
    }

    Ok(())
}
