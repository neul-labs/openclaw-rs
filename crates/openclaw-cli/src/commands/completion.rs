//! Shell completion command.

use crate::ui;
use anyhow::Result;
use std::path::PathBuf;

/// Completion command arguments.
#[derive(Debug, Clone)]
pub struct CompletionArgs {
    /// Shell to generate completions for.
    pub shell: Option<String>,
    /// Install completions to shell profile.
    pub install: bool,
    /// Write completion state cache.
    pub write_state: bool,
}

impl Default for CompletionArgs {
    fn default() -> Self {
        Self {
            shell: None,
            install: false,
            write_state: false,
        }
    }
}

/// Run the completion command.
pub async fn run_completion(args: CompletionArgs) -> Result<()> {
    let shell = args.shell.as_deref().unwrap_or_else(|| detect_shell());

    if args.write_state {
        write_completion_cache(shell)?;
        ui::success(&format!("Completion cache written for {}", shell));
        return Ok(());
    }

    if args.install {
        install_completion(shell)?;
        return Ok(());
    }

    // Print completion script to stdout
    let script = generate_completion(shell)?;
    println!("{}", script);

    Ok(())
}

/// Detect the current shell.
fn detect_shell() -> &'static str {
    if let Ok(shell) = std::env::var("SHELL") {
        if shell.contains("zsh") {
            return "zsh";
        }
        if shell.contains("bash") {
            return "bash";
        }
        if shell.contains("fish") {
            return "fish";
        }
    }

    // Default to bash
    "bash"
}

/// Generate completion script for a shell.
fn generate_completion(shell: &str) -> Result<String> {
    match shell {
        "zsh" => Ok(generate_zsh_completion()),
        "bash" => Ok(generate_bash_completion()),
        "fish" => Ok(generate_fish_completion()),
        "powershell" | "pwsh" => Ok(generate_powershell_completion()),
        _ => anyhow::bail!("Unsupported shell: {}", shell),
    }
}

/// Write completion cache to state directory.
fn write_completion_cache(shell: &str) -> Result<()> {
    let completion_dir = get_completion_dir();
    std::fs::create_dir_all(&completion_dir)?;

    let script = generate_completion(shell)?;
    let filename = match shell {
        "zsh" => "openclaw.zsh",
        "bash" => "openclaw.bash",
        "fish" => "openclaw.fish",
        "powershell" | "pwsh" => "openclaw.ps1",
        _ => anyhow::bail!("Unsupported shell: {}", shell),
    };

    let path = completion_dir.join(filename);
    std::fs::write(&path, script)?;

    ui::info(&format!("Wrote completion to {}", path.display()));

    Ok(())
}

/// Install completion to shell profile.
fn install_completion(shell: &str) -> Result<()> {
    // First write the cache
    write_completion_cache(shell)?;

    let completion_dir = get_completion_dir();
    let source_line = match shell {
        "zsh" => format!(
            "\n# OpenClaw completion\nsource \"{}/openclaw.zsh\"\n",
            completion_dir.display()
        ),
        "bash" => format!(
            "\n# OpenClaw completion\nsource \"{}/openclaw.bash\"\n",
            completion_dir.display()
        ),
        "fish" => format!(
            "\n# OpenClaw completion\nsource \"{}/openclaw.fish\"\n",
            completion_dir.display()
        ),
        "powershell" | "pwsh" => format!(
            "\n# OpenClaw completion\n. \"{}/openclaw.ps1\"\n",
            completion_dir.display()
        ),
        _ => anyhow::bail!("Unsupported shell: {}", shell),
    };

    // Get profile path
    let profile_path = get_profile_path(shell)?;

    // Check if already installed
    if let Ok(content) = std::fs::read_to_string(&profile_path) {
        if content.contains("OpenClaw completion") {
            ui::info("Completion already installed");
            return Ok(());
        }
    }

    // Append to profile
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&profile_path)?;

    use std::io::Write;
    file.write_all(source_line.as_bytes())?;

    ui::success(&format!(
        "Completion installed. Restart your shell or run:\n  source {}",
        profile_path.display()
    ));

    Ok(())
}

/// Get the completion directory.
fn get_completion_dir() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join(".openclaw").join("completions"))
        .unwrap_or_else(|| PathBuf::from(".openclaw/completions"))
}

/// Get the shell profile path.
fn get_profile_path(shell: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;

    Ok(match shell {
        "zsh" => home.join(".zshrc"),
        "bash" => {
            // Prefer .bashrc, fall back to .bash_profile
            let bashrc = home.join(".bashrc");
            if bashrc.exists() {
                bashrc
            } else {
                home.join(".bash_profile")
            }
        }
        "fish" => home.join(".config/fish/config.fish"),
        "powershell" | "pwsh" => {
            // PowerShell profile location varies by platform
            #[cfg(windows)]
            {
                home.join("Documents/PowerShell/Microsoft.PowerShell_profile.ps1")
            }
            #[cfg(not(windows))]
            {
                home.join(".config/powershell/Microsoft.PowerShell_profile.ps1")
            }
        }
        _ => anyhow::bail!("Unsupported shell: {}", shell),
    })
}

/// Generate Zsh completion script.
fn generate_zsh_completion() -> String {
    r#"#compdef openclaw

_openclaw() {
    local -a commands
    commands=(
        'onboard:Run the onboarding wizard'
        'configure:Update configuration interactively'
        'doctor:Run health checks and auto-repair'
        'status:Show gateway and channel status'
        'gateway:Gateway operations'
        'channels:Channel management'
        'config:Configuration get/set'
        'completion:Shell completion setup'
        'daemon:Daemon management'
        'reset:Reset configuration'
        'help:Show help'
    )

    local -a gateway_commands
    gateway_commands=(
        'run:Start the gateway server'
        'status:Check gateway status'
    )

    local -a daemon_commands
    daemon_commands=(
        'install:Install as system service'
        'uninstall:Remove system service'
        'start:Start the daemon'
        'stop:Stop the daemon'
        'status:Check daemon status'
    )

    _arguments -C \
        '(-v --verbose)'{-v,--verbose}'[Verbose output]' \
        '(-h --help)'{-h,--help}'[Show help]' \
        '1: :->command' \
        '*:: :->args'

    case $state in
        command)
            _describe -t commands 'openclaw command' commands
            ;;
        args)
            case $words[1] in
                gateway)
                    _describe -t commands 'gateway command' gateway_commands
                    ;;
                daemon)
                    _describe -t commands 'daemon command' daemon_commands
                    ;;
                onboard)
                    _arguments \
                        '--non-interactive[Non-interactive mode]' \
                        '--accept-risk[Accept security risks]' \
                        '--flow[Setup flow (quickstart, advanced)]:flow:(quickstart advanced)' \
                        '--install-daemon[Install daemon after setup]'
                    ;;
                doctor)
                    _arguments \
                        '--repair[Apply recommended fixes]' \
                        '--force[Aggressive repairs]' \
                        '--deep[Deep scan]'
                    ;;
                completion)
                    _arguments \
                        '--shell[Shell type]:shell:(zsh bash fish powershell)' \
                        '--install[Install to profile]' \
                        '--write-state[Write completion cache]'
                    ;;
            esac
            ;;
    esac
}

_openclaw "$@"
"#
    .to_string()
}

/// Generate Bash completion script.
fn generate_bash_completion() -> String {
    r#"_openclaw() {
    local cur prev commands
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    commands="onboard configure doctor status gateway channels config completion daemon reset help"

    case "${prev}" in
        openclaw)
            COMPREPLY=( $(compgen -W "${commands}" -- ${cur}) )
            return 0
            ;;
        gateway)
            COMPREPLY=( $(compgen -W "run status" -- ${cur}) )
            return 0
            ;;
        daemon)
            COMPREPLY=( $(compgen -W "install uninstall start stop status" -- ${cur}) )
            return 0
            ;;
        --shell)
            COMPREPLY=( $(compgen -W "zsh bash fish powershell" -- ${cur}) )
            return 0
            ;;
        --flow)
            COMPREPLY=( $(compgen -W "quickstart advanced" -- ${cur}) )
            return 0
            ;;
        *)
            ;;
    esac

    if [[ ${cur} == -* ]]; then
        COMPREPLY=( $(compgen -W "--help --verbose" -- ${cur}) )
        return 0
    fi
}

complete -F _openclaw openclaw
"#
    .to_string()
}

/// Generate Fish completion script.
fn generate_fish_completion() -> String {
    r#"complete -c openclaw -f

# Main commands
complete -c openclaw -n "__fish_use_subcommand" -a onboard -d "Run the onboarding wizard"
complete -c openclaw -n "__fish_use_subcommand" -a configure -d "Update configuration"
complete -c openclaw -n "__fish_use_subcommand" -a doctor -d "Run health checks"
complete -c openclaw -n "__fish_use_subcommand" -a status -d "Show status"
complete -c openclaw -n "__fish_use_subcommand" -a gateway -d "Gateway operations"
complete -c openclaw -n "__fish_use_subcommand" -a channels -d "Channel management"
complete -c openclaw -n "__fish_use_subcommand" -a config -d "Configuration"
complete -c openclaw -n "__fish_use_subcommand" -a completion -d "Shell completion"
complete -c openclaw -n "__fish_use_subcommand" -a daemon -d "Daemon management"
complete -c openclaw -n "__fish_use_subcommand" -a reset -d "Reset configuration"

# Gateway subcommands
complete -c openclaw -n "__fish_seen_subcommand_from gateway" -a run -d "Start gateway"
complete -c openclaw -n "__fish_seen_subcommand_from gateway" -a status -d "Gateway status"

# Daemon subcommands
complete -c openclaw -n "__fish_seen_subcommand_from daemon" -a install -d "Install daemon"
complete -c openclaw -n "__fish_seen_subcommand_from daemon" -a uninstall -d "Uninstall daemon"
complete -c openclaw -n "__fish_seen_subcommand_from daemon" -a start -d "Start daemon"
complete -c openclaw -n "__fish_seen_subcommand_from daemon" -a stop -d "Stop daemon"
complete -c openclaw -n "__fish_seen_subcommand_from daemon" -a status -d "Daemon status"

# Global options
complete -c openclaw -s v -l verbose -d "Verbose output"
complete -c openclaw -s h -l help -d "Show help"
"#
    .to_string()
}

/// Generate PowerShell completion script.
fn generate_powershell_completion() -> String {
    r#"Register-ArgumentCompleter -Native -CommandName openclaw -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commands = @(
        @{ Name = 'onboard'; Description = 'Run the onboarding wizard' }
        @{ Name = 'configure'; Description = 'Update configuration' }
        @{ Name = 'doctor'; Description = 'Run health checks' }
        @{ Name = 'status'; Description = 'Show status' }
        @{ Name = 'gateway'; Description = 'Gateway operations' }
        @{ Name = 'channels'; Description = 'Channel management' }
        @{ Name = 'config'; Description = 'Configuration' }
        @{ Name = 'completion'; Description = 'Shell completion' }
        @{ Name = 'daemon'; Description = 'Daemon management' }
        @{ Name = 'reset'; Description = 'Reset configuration' }
    )

    $commands | Where-Object { $_.Name -like "$wordToComplete*" } | ForEach-Object {
        [System.Management.Automation.CompletionResult]::new($_.Name, $_.Name, 'ParameterValue', $_.Description)
    }
}
"#
    .to_string()
}
