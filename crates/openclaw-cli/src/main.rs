//! OpenClaw CLI - Command-line interface for OpenClaw.

mod commands;
mod ui;

use clap::{Parser, Subcommand};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Parser)]
#[command(name = "openclaw")]
#[command(about = "OpenClaw - AI agent platform")]
#[command(version)]
#[command(propagate_version = true)]
struct Cli {
    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the interactive onboarding wizard
    Onboard {
        /// Non-interactive mode
        #[arg(long)]
        non_interactive: bool,

        /// Accept risk acknowledgement (required for non-interactive)
        #[arg(long)]
        accept_risk: bool,

        /// Setup flow: quickstart or advanced
        #[arg(long)]
        flow: Option<String>,

        /// Auth provider choice
        #[arg(long)]
        auth_choice: Option<String>,

        /// API key for selected provider
        #[arg(long)]
        api_key: Option<String>,

        /// Install daemon after setup
        #[arg(long)]
        install_daemon: bool,
    },

    /// Update configuration interactively
    Configure {
        /// Section to configure: auth, gateway, agents, channels, workspace
        #[arg(long)]
        section: Option<String>,
    },

    /// Run health checks and auto-repair
    Doctor {
        /// Apply recommended fixes
        #[arg(long, alias = "fix")]
        repair: bool,

        /// Aggressive repairs (may overwrite custom config)
        #[arg(long)]
        force: bool,

        /// Deep scan for extra service installs
        #[arg(long)]
        deep: bool,
    },

    /// Show gateway, agents, and channels status
    Status {
        /// Show all details
        #[arg(long)]
        all: bool,

        /// Probe services for connectivity
        #[arg(long)]
        deep: bool,
    },

    /// Gateway operations
    Gateway {
        #[command(subcommand)]
        action: GatewayCommands,
    },

    /// Channel management
    Channels {
        /// List configured channels
        #[arg(long)]
        list: bool,

        /// Probe channels for connectivity
        #[arg(long)]
        probe: bool,
    },

    /// Configuration get/set
    Config {
        #[command(subcommand)]
        action: Option<ConfigCommands>,
    },

    /// Shell completion setup
    Completion {
        /// Shell type: zsh, bash, fish, powershell
        #[arg(long)]
        shell: Option<String>,

        /// Install completions to shell profile
        #[arg(long)]
        install: bool,

        /// Write completion state cache
        #[arg(long)]
        write_state: bool,
    },

    /// Daemon management (system service)
    Daemon {
        #[command(subcommand)]
        action: DaemonCommands,
    },

    /// Reset configuration or state
    Reset {
        /// Reset only configuration
        #[arg(long)]
        config_only: bool,

        /// Reset everything including sessions
        #[arg(long)]
        all: bool,
    },

    /// User management (admin commands)
    Admin {
        #[command(subcommand)]
        action: AdminCommands,

        /// Data directory override
        #[arg(long, global = true)]
        data_dir: Option<std::path::PathBuf>,
    },
}

#[derive(Subcommand)]
enum GatewayCommands {
    /// Start the gateway server
    Run {
        /// Port to listen on
        #[arg(short, long)]
        port: Option<u16>,

        /// Bind address (loopback, lan, or IP)
        #[arg(long)]
        bind: Option<String>,

        /// Force start even if port is in use
        #[arg(long)]
        force: bool,
    },

    /// Check gateway status
    Status,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Get a configuration value
    Get {
        /// Configuration key (e.g., gateway.port)
        key: String,
    },

    /// Set a configuration value
    Set {
        /// Configuration key (e.g., gateway.port)
        key: String,

        /// Value to set
        value: String,
    },

    /// Show full configuration
    Show,

    /// Validate configuration
    Validate,
}

#[derive(Subcommand)]
enum DaemonCommands {
    /// Install as system service
    Install,

    /// Remove system service
    Uninstall,

    /// Start the daemon
    Start,

    /// Stop the daemon
    Stop,

    /// Check daemon status
    Status,
}

#[derive(Subcommand)]
enum AdminCommands {
    /// Create a new user
    Create {
        /// Username for the new user
        #[arg(long)]
        username: String,

        /// Password (or use --generate-password)
        #[arg(long)]
        password: Option<String>,

        /// User role: admin, operator, or viewer
        #[arg(long, default_value = "admin")]
        role: String,

        /// Generate a random password
        #[arg(long)]
        generate_password: bool,
    },

    /// List all users
    List,

    /// Reset a user's password
    ResetPassword {
        /// Username of the user
        #[arg(long)]
        username: String,
    },

    /// Enable a user account
    Enable {
        /// Username of the user
        #[arg(long)]
        username: String,
    },

    /// Disable a user account
    Disable {
        /// Username of the user
        #[arg(long)]
        username: String,
    },

    /// Delete a user
    Delete {
        /// Username of the user to delete
        #[arg(long)]
        username: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false))
        .with(filter)
        .init();

    // If no command, show help or run onboard for first-time users
    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            // Check if this is first run
            let config_exists = openclaw_core::Config::load_default().is_ok();

            if config_exists {
                // Show status by default
                commands::run_status(commands::status::StatusArgs::default()).await?;
                return Ok(());
            } else {
                // First run - suggest onboarding
                ui::banner();
                ui::info("Welcome to OpenClaw!");
                ui::info("Run 'openclaw onboard' to get started, or 'openclaw --help' for all commands.");
                return Ok(());
            }
        }
    };

    match command {
        Commands::Onboard {
            non_interactive,
            accept_risk,
            flow,
            auth_choice,
            api_key,
            install_daemon,
        } => {
            let args = commands::onboard::OnboardArgs {
                non_interactive,
                accept_risk,
                flow,
                auth_choice,
                api_key,
                install_daemon,
            };
            commands::run_onboard(args).await?;
        }

        Commands::Configure { section } => {
            let args = commands::configure::ConfigureArgs { section };
            commands::run_configure(args).await?;
        }

        Commands::Doctor {
            repair,
            force,
            deep,
        } => {
            let args = commands::doctor::DoctorArgs {
                repair,
                force,
                deep,
            };
            commands::run_doctor(args).await?;
        }

        Commands::Status { all, deep } => {
            let args = commands::status::StatusArgs { all, deep };
            commands::run_status(args).await?;
        }

        Commands::Gateway { action } => {
            let args = match action {
                GatewayCommands::Run { port, bind, force } => commands::gateway::GatewayArgs {
                    action: commands::gateway::GatewayAction::Run { port, bind, force },
                },
                GatewayCommands::Status => commands::gateway::GatewayArgs {
                    action: commands::gateway::GatewayAction::Status,
                },
            };
            commands::run_gateway(args).await?;
        }

        Commands::Channels { list: _, probe } => {
            if probe {
                ui::info("Probing channels...");
                // TODO: Implement channel probing
                ui::warning("Channel probing not yet implemented");
            } else {
                ui::info("Listing channels...");
                // TODO: Implement channel listing
                ui::warning("Channel listing not yet implemented");
            }
        }

        Commands::Config { action } => {
            let args = match action {
                Some(ConfigCommands::Get { key }) => commands::config::ConfigArgs {
                    get: Some(key),
                    set: None,
                    show: false,
                    validate: false,
                },
                Some(ConfigCommands::Set { key, value }) => commands::config::ConfigArgs {
                    get: None,
                    set: Some(format!("{}={}", key, value)),
                    show: false,
                    validate: false,
                },
                Some(ConfigCommands::Show) => commands::config::ConfigArgs {
                    get: None,
                    set: None,
                    show: true,
                    validate: false,
                },
                Some(ConfigCommands::Validate) => commands::config::ConfigArgs {
                    get: None,
                    set: None,
                    show: false,
                    validate: true,
                },
                None => commands::config::ConfigArgs {
                    get: None,
                    set: None,
                    show: true,
                    validate: false,
                },
            };
            commands::run_config(args).await?;
        }

        Commands::Completion {
            shell,
            install,
            write_state,
        } => {
            let args = commands::completion::CompletionArgs {
                shell,
                install,
                write_state,
            };
            commands::run_completion(args).await?;
        }

        Commands::Daemon { action } => {
            let args = commands::daemon::DaemonArgs {
                action: match action {
                    DaemonCommands::Install => commands::daemon::DaemonAction::Install,
                    DaemonCommands::Uninstall => commands::daemon::DaemonAction::Uninstall,
                    DaemonCommands::Start => commands::daemon::DaemonAction::Start,
                    DaemonCommands::Stop => commands::daemon::DaemonAction::Stop,
                    DaemonCommands::Status => commands::daemon::DaemonAction::Status,
                },
            };
            commands::run_daemon(args).await?;
        }

        Commands::Reset { config_only, all } => {
            ui::header("Reset OpenClaw");

            if all {
                ui::warning("This will delete all OpenClaw data including sessions!");
            } else if config_only {
                ui::warning("This will delete your configuration file.");
            } else {
                ui::info("This will reset configuration and credentials.");
            }

            let confirm = ui::prompts::confirm("Are you sure you want to continue?")?;

            if confirm {
                let state_dir = dirs::home_dir()
                    .map(|h| h.join(".openclaw"))
                    .unwrap_or_default();

                if all {
                    if state_dir.exists() {
                        std::fs::remove_dir_all(&state_dir)?;
                        ui::success("All OpenClaw data deleted");
                    }
                } else if config_only {
                    let config_path = state_dir.join("openclaw.json");
                    if config_path.exists() {
                        std::fs::remove_file(&config_path)?;
                        ui::success("Configuration deleted");
                    }
                } else {
                    let config_path = state_dir.join("openclaw.json");
                    let cred_path = state_dir.join("credentials");

                    if config_path.exists() {
                        std::fs::remove_file(&config_path)?;
                    }
                    if cred_path.exists() {
                        std::fs::remove_dir_all(&cred_path)?;
                    }

                    ui::success("Configuration and credentials deleted");
                }

                ui::info("Run 'openclaw onboard' to set up again");
            } else {
                ui::info("Reset cancelled");
            }
        }

        Commands::Admin { action, data_dir } => {
            let args = commands::admin::AdminArgs {
                action: match action {
                    AdminCommands::Create {
                        username,
                        password,
                        role,
                        generate_password,
                    } => commands::admin::AdminAction::Create {
                        username,
                        password,
                        role,
                        generate_password,
                    },
                    AdminCommands::List => commands::admin::AdminAction::List,
                    AdminCommands::ResetPassword { username } => {
                        commands::admin::AdminAction::ResetPassword { username }
                    }
                    AdminCommands::Enable { username } => {
                        commands::admin::AdminAction::Enable { username }
                    }
                    AdminCommands::Disable { username } => {
                        commands::admin::AdminAction::Disable { username }
                    }
                    AdminCommands::Delete { username } => {
                        commands::admin::AdminAction::Delete { username }
                    }
                },
                data_dir,
            };
            commands::run_admin(args).await?;
        }
    }

    Ok(())
}
