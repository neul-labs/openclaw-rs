//! Formatted output utilities.

use console::{style, Style, Term};

/// Print a success message with checkmark.
pub fn success(msg: &str) {
    println!("{} {}", style("✓").green().bold(), msg);
}

/// Print an error message with X.
pub fn error(msg: &str) {
    println!("{} {}", style("✗").red().bold(), msg);
}

/// Print a warning message.
pub fn warning(msg: &str) {
    println!("{} {}", style("⚠").yellow().bold(), msg);
}

/// Print an info message.
pub fn info(msg: &str) {
    println!("{} {}", style("ℹ").blue().bold(), msg);
}

/// Print a header/section title.
pub fn header(msg: &str) {
    println!("\n{}", style(msg).bold().underlined());
}

/// Print a step in a process.
pub fn step(num: usize, total: usize, msg: &str) {
    println!(
        "{} {}",
        style(format!("[{}/{}]", num, total)).dim(),
        msg
    );
}

/// Health check result display.
pub fn health_check(name: &str, status: HealthStatus, detail: Option<&str>) {
    let (icon, status_style) = match status {
        HealthStatus::Ok => (style("✓").green(), Style::new().green()),
        HealthStatus::Warning => (style("⚠").yellow(), Style::new().yellow()),
        HealthStatus::Error => (style("✗").red(), Style::new().red()),
        HealthStatus::Unknown => (style("?").dim(), Style::new().dim()),
    };

    let status_text = match status {
        HealthStatus::Ok => "OK",
        HealthStatus::Warning => "WARNING",
        HealthStatus::Error => "ERROR",
        HealthStatus::Unknown => "UNKNOWN",
    };

    print!("  {} {}: ", icon, name);
    print!("{}", status_style.apply_to(status_text));

    if let Some(d) = detail {
        print!(" - {}", style(d).dim());
    }
    println!();
}

/// Health check status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Ok,
    Warning,
    Error,
    Unknown,
}

/// Clear the terminal.
pub fn clear() {
    let _ = Term::stdout().clear_screen();
}

/// Print the OpenClaw banner.
pub fn banner() {
    println!(
        "{}",
        style(
            r"
   ____                   ____  _
  / __ \                 / ___|| |
 | |  | |_ __   ___ _ __ | |    | | __ ___      __
 | |  | | '_ \ / _ | '_ \| |    | |/ _` \ \ /\ / /
 | |__| | |_) |  __| | | | |___ | | (_| |\ V  V /
  \____/| .__/ \___|_| |_|\____||_|\__,_| \_/\_/
        | |
        |_|
"
        )
        .cyan()
    );
}

/// Print a key-value pair.
pub fn kv(key: &str, value: &str) {
    println!("  {}: {}", style(key).bold(), value);
}

/// Print a table row.
pub fn table_row(cols: &[(&str, usize)]) {
    for (text, width) in cols {
        print!("{:width$}", text, width = width);
    }
    println!();
}
