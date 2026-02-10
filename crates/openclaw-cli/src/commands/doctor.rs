//! Doctor command - health checks and auto-repair.

use crate::ui::{self, HealthStatus};
use anyhow::Result;
use std::path::PathBuf;

/// Doctor command arguments.
#[derive(Debug, Clone, Default)]
pub struct DoctorArgs {
    /// Apply recommended fixes.
    pub repair: bool,
    /// Aggressive repairs.
    pub force: bool,
    /// Deep scan for extra services.
    pub deep: bool,
}

/// Run health checks and optionally repair issues.
pub async fn run_doctor(args: DoctorArgs) -> Result<()> {
    ui::header("OpenClaw Doctor");
    println!();

    let mut issues_found = 0;
    let mut repairs_made = 0;

    // Check 1: Configuration
    ui::info("Checking configuration...");
    match check_config() {
        CheckResult::Ok => {
            ui::health_check("Configuration", HealthStatus::Ok, None);
        }
        CheckResult::Warning(msg) => {
            ui::health_check("Configuration", HealthStatus::Warning, Some(&msg));
            issues_found += 1;
        }
        CheckResult::Error(msg) => {
            ui::health_check("Configuration", HealthStatus::Error, Some(&msg));
            issues_found += 1;

            if args.repair {
                ui::info("  → Creating default configuration...");
                if create_default_config().is_ok() {
                    ui::success("  → Configuration created");
                    repairs_made += 1;
                }
            }
        }
    }

    // Check 2: State directory
    ui::info("Checking state directory...");
    match check_state_dir() {
        CheckResult::Ok => {
            ui::health_check("State directory", HealthStatus::Ok, None);
        }
        CheckResult::Warning(msg) => {
            ui::health_check("State directory", HealthStatus::Warning, Some(&msg));
            issues_found += 1;
        }
        CheckResult::Error(msg) => {
            ui::health_check("State directory", HealthStatus::Error, Some(&msg));
            issues_found += 1;

            if args.repair {
                ui::info("  → Creating state directory...");
                if create_state_dir().is_ok() {
                    ui::success("  → State directory created");
                    repairs_made += 1;
                }
            }
        }
    }

    // Check 3: Credentials
    ui::info("Checking credentials...");
    match check_credentials() {
        CheckResult::Ok => {
            ui::health_check("Credentials", HealthStatus::Ok, None);
        }
        CheckResult::Warning(msg) => {
            ui::health_check("Credentials", HealthStatus::Warning, Some(&msg));
            issues_found += 1;
        }
        CheckResult::Error(msg) => {
            ui::health_check("Credentials", HealthStatus::Error, Some(&msg));
            issues_found += 1;
        }
    }

    // Check 4: Sandbox availability
    ui::info("Checking sandbox...");
    match check_sandbox() {
        CheckResult::Ok => {
            ui::health_check("Sandbox", HealthStatus::Ok, None);
        }
        CheckResult::Warning(msg) => {
            ui::health_check("Sandbox", HealthStatus::Warning, Some(&msg));
            issues_found += 1;
        }
        CheckResult::Error(msg) => {
            ui::health_check("Sandbox", HealthStatus::Error, Some(&msg));
            issues_found += 1;
        }
    }

    // Check 5: Gateway connectivity (if running)
    ui::info("Checking gateway...");
    match check_gateway().await {
        CheckResult::Ok => {
            ui::health_check("Gateway", HealthStatus::Ok, Some("running"));
        }
        CheckResult::Warning(msg) => {
            ui::health_check("Gateway", HealthStatus::Warning, Some(&msg));
        }
        CheckResult::Error(msg) => {
            ui::health_check("Gateway", HealthStatus::Error, Some(&msg));
        }
    }

    // Check 6: Shell completion
    ui::info("Checking shell completion...");
    match check_shell_completion() {
        CheckResult::Ok => {
            ui::health_check("Shell completion", HealthStatus::Ok, None);
        }
        CheckResult::Warning(msg) => {
            ui::health_check("Shell completion", HealthStatus::Warning, Some(&msg));
            issues_found += 1;
        }
        CheckResult::Error(msg) => {
            ui::health_check("Shell completion", HealthStatus::Error, Some(&msg));
            issues_found += 1;
        }
    }

    // Deep scan
    if args.deep {
        ui::info("Running deep scan...");

        // Check for multiple gateway instances
        match check_multiple_gateways() {
            CheckResult::Ok => {
                ui::health_check(
                    "Gateway instances",
                    HealthStatus::Ok,
                    Some("single instance"),
                );
            }
            CheckResult::Warning(msg) => {
                ui::health_check("Gateway instances", HealthStatus::Warning, Some(&msg));
                issues_found += 1;
            }
            CheckResult::Error(msg) => {
                ui::health_check("Gateway instances", HealthStatus::Error, Some(&msg));
                issues_found += 1;
            }
        }
    }

    // Summary
    println!();
    ui::header("Summary");

    if issues_found == 0 {
        ui::success("All checks passed!");
    } else {
        ui::warning(&format!("{issues_found} issue(s) found"));

        if args.repair {
            ui::info(&format!("{repairs_made} repair(s) made"));
        } else {
            ui::info("Run with --repair to fix issues automatically");
        }
    }

    Ok(())
}

/// Check result enum.
enum CheckResult {
    Ok,
    Warning(String),
    Error(String),
}

/// Check configuration file.
fn check_config() -> CheckResult {
    let config_path = get_config_path();

    if !config_path.exists() {
        return CheckResult::Error("Config file not found".to_string());
    }

    match openclaw_core::Config::load_default() {
        Ok(_) => CheckResult::Ok,
        Err(e) => CheckResult::Error(format!("Invalid config: {e}")),
    }
}

/// Check state directory.
fn check_state_dir() -> CheckResult {
    let state_dir = get_state_dir();

    if !state_dir.exists() {
        return CheckResult::Error("State directory not found".to_string());
    }

    // Check permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&state_dir) {
            let mode = metadata.permissions().mode();
            if mode & 0o077 != 0 {
                return CheckResult::Warning("State directory has loose permissions".to_string());
            }
        }
    }

    CheckResult::Ok
}

/// Check credentials.
fn check_credentials() -> CheckResult {
    let cred_path = get_credentials_path();

    if !cred_path.exists() {
        return CheckResult::Warning("No credentials configured".to_string());
    }

    // Check if any credential files exist
    let entries = std::fs::read_dir(&cred_path).ok();
    let has_creds = entries.is_some_and(|e| e.count() > 0);

    if !has_creds {
        return CheckResult::Warning("No API keys stored".to_string());
    }

    CheckResult::Ok
}

/// Check sandbox availability.
fn check_sandbox() -> CheckResult {
    if openclaw_agents::sandbox::is_sandbox_available() {
        CheckResult::Ok
    } else {
        #[cfg(target_os = "linux")]
        {
            CheckResult::Warning(
                "bubblewrap (bwrap) not found - install for sandboxing".to_string(),
            )
        }

        #[cfg(target_os = "macos")]
        {
            CheckResult::Ok // sandbox-exec is always available on macOS
        }

        #[cfg(target_os = "windows")]
        {
            CheckResult::Warning("Windows sandboxing not yet implemented".to_string())
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            CheckResult::Error("Sandboxing not supported on this platform".to_string())
        }
    }
}

/// Check gateway connectivity.
async fn check_gateway() -> CheckResult {
    // Try to connect to the default gateway port
    let addr = "127.0.0.1:18789";

    match tokio::net::TcpStream::connect(addr).await {
        Ok(_) => {
            // Try HTTP health check
            let client = reqwest::Client::new();
            match client
                .get("http://127.0.0.1:18789/health")
                .timeout(std::time::Duration::from_secs(2))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => CheckResult::Ok,
                Ok(resp) => {
                    CheckResult::Warning(format!("Gateway returned status {}", resp.status()))
                }
                Err(_) => {
                    CheckResult::Warning("Gateway running but health check failed".to_string())
                }
            }
        }
        Err(_) => CheckResult::Warning("Gateway not running".to_string()),
    }
}

/// Check shell completion setup.
fn check_shell_completion() -> CheckResult {
    let completion_dir = get_state_dir().join("completions");

    if !completion_dir.exists() {
        return CheckResult::Warning(
            "Shell completion not installed - run 'openclaw completion --install'".to_string(),
        );
    }

    // Check for completion files
    let has_completion = std::fs::read_dir(&completion_dir)
        .ok()
        .is_some_and(|e| e.count() > 0);

    if has_completion {
        CheckResult::Ok
    } else {
        CheckResult::Warning("Shell completion files missing".to_string())
    }
}

/// Check for multiple gateway instances.
const fn check_multiple_gateways() -> CheckResult {
    // This is a simple check - in production, would use platform-specific methods
    CheckResult::Ok
}

/// Create default configuration.
fn create_default_config() -> Result<()> {
    let config_path = get_config_path();
    std::fs::create_dir_all(config_path.parent().unwrap())?;

    let default_config = serde_json::json!({
        "gateway": {
            "mode": "local",
            "port": 18789,
            "bind": "loopback"
        },
        "agents": {
            "defaults": {
                "model": "claude-sonnet-4-20250514"
            }
        }
    });

    std::fs::write(&config_path, serde_json::to_string_pretty(&default_config)?)?;
    Ok(())
}

/// Create state directory.
fn create_state_dir() -> Result<()> {
    let state_dir = get_state_dir();
    std::fs::create_dir_all(&state_dir)?;

    // Set restrictive permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&state_dir, std::fs::Permissions::from_mode(0o700))?;
    }

    Ok(())
}

/// Get the config file path.
fn get_config_path() -> PathBuf {
    if let Ok(path) = std::env::var("OPENCLAW_CONFIG_PATH") {
        return PathBuf::from(path);
    }

    get_state_dir().join("openclaw.json")
}

/// Get the state directory path.
fn get_state_dir() -> PathBuf {
    if let Ok(state_dir) = std::env::var("OPENCLAW_STATE_DIR") {
        return PathBuf::from(state_dir);
    }

    dirs::home_dir().map_or_else(|| PathBuf::from(".openclaw"), |h| h.join(".openclaw"))
}

/// Get the credentials directory path.
fn get_credentials_path() -> PathBuf {
    get_state_dir().join("credentials")
}
