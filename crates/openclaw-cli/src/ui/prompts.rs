//! Interactive prompt utilities.

use console::style;
use dialoguer::{Confirm, Input, Password, Select, theme::ColorfulTheme};

/// Get the default colorful theme.
fn theme() -> ColorfulTheme {
    ColorfulTheme::default()
}

/// Prompt for text input.
pub fn input(prompt: &str) -> Result<String, dialoguer::Error> {
    Input::with_theme(&theme())
        .with_prompt(prompt)
        .interact_text()
}

/// Prompt for text input with a default value.
pub fn input_with_default(prompt: &str, default: &str) -> Result<String, dialoguer::Error> {
    Input::with_theme(&theme())
        .with_prompt(prompt)
        .default(default.to_string())
        .interact_text()
}

/// Prompt for optional text input.
pub fn input_optional(prompt: &str) -> Result<Option<String>, dialoguer::Error> {
    let result: String = Input::with_theme(&theme())
        .with_prompt(prompt)
        .allow_empty(true)
        .interact_text()?;

    if result.is_empty() {
        Ok(None)
    } else {
        Ok(Some(result))
    }
}

/// Prompt for a password (hidden input).
pub fn password(prompt: &str) -> Result<String, dialoguer::Error> {
    Password::with_theme(&theme())
        .with_prompt(prompt)
        .interact()
}

/// Prompt for confirmation (yes/no).
pub fn confirm(prompt: &str) -> Result<bool, dialoguer::Error> {
    Confirm::with_theme(&theme())
        .with_prompt(prompt)
        .default(false)
        .interact()
}

/// Prompt for confirmation with a default of true.
#[allow(dead_code)]
pub fn confirm_default_yes(prompt: &str) -> Result<bool, dialoguer::Error> {
    Confirm::with_theme(&theme())
        .with_prompt(prompt)
        .default(true)
        .interact()
}

/// Prompt for selection from a list of options.
#[allow(dead_code)]
pub fn select<T: ToString>(prompt: &str, options: &[T]) -> Result<usize, dialoguer::Error> {
    Select::with_theme(&theme())
        .with_prompt(prompt)
        .items(options)
        .default(0)
        .interact()
}

/// Prompt for selection with descriptions.
pub fn select_with_help(prompt: &str, options: &[(&str, &str)]) -> Result<usize, dialoguer::Error> {
    let items: Vec<String> = options
        .iter()
        .map(|(name, desc)| format!("{} - {}", style(name).bold(), style(desc).dim()))
        .collect();

    Select::with_theme(&theme())
        .with_prompt(prompt)
        .items(&items)
        .default(0)
        .interact()
}

/// Print a risk acknowledgement prompt for onboarding.
pub fn risk_acknowledgement() -> Result<bool, dialoguer::Error> {
    println!();
    println!("{}", style("⚠️  Important Security Notice").yellow().bold());
    println!();
    println!("OpenClaw is a powerful AI agent platform that can:");
    println!("  • Execute arbitrary code on your system");
    println!("  • Access files and network resources");
    println!("  • Interact with external services");
    println!();
    println!(
        "For more information, see: {}",
        style("https://docs.openclaw.ai/security")
            .cyan()
            .underlined()
    );
    println!();

    Confirm::with_theme(&theme())
        .with_prompt("I understand and accept the risks")
        .default(false)
        .interact()
}

/// Flow selection for onboarding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnboardFlow {
    QuickStart,
    Advanced,
}

impl std::fmt::Display for OnboardFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::QuickStart => write!(f, "QuickStart (recommended)"),
            Self::Advanced => write!(f, "Advanced"),
        }
    }
}

/// Prompt for onboarding flow selection.
pub fn select_onboard_flow() -> Result<OnboardFlow, dialoguer::Error> {
    let options = [
        ("QuickStart", "Pre-configured defaults, minimal prompts"),
        ("Advanced", "Full control over all settings"),
    ];

    let selection = select_with_help("Select setup mode", &options)?;

    Ok(match selection {
        0 => OnboardFlow::QuickStart,
        _ => OnboardFlow::Advanced,
    })
}

/// Auth provider selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthProvider {
    Anthropic,
    OpenAI,
    OpenRouter,
    Skip,
}

impl std::fmt::Display for AuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Anthropic => write!(f, "Anthropic (Claude)"),
            Self::OpenAI => write!(f, "OpenAI (GPT)"),
            Self::OpenRouter => write!(f, "OpenRouter"),
            Self::Skip => write!(f, "Skip for now"),
        }
    }
}

/// Prompt for auth provider selection.
pub fn select_auth_provider() -> Result<AuthProvider, dialoguer::Error> {
    let options = [
        ("Anthropic", "Claude models (recommended)"),
        ("OpenAI", "GPT models"),
        ("OpenRouter", "Multiple providers via OpenRouter"),
        ("Skip", "Configure later"),
    ];

    let selection = select_with_help("Select AI provider", &options)?;

    Ok(match selection {
        0 => AuthProvider::Anthropic,
        1 => AuthProvider::OpenAI,
        2 => AuthProvider::OpenRouter,
        _ => AuthProvider::Skip,
    })
}
