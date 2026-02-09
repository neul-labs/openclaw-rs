//! Config get/set command.

use crate::ui;
use anyhow::Result;
use std::path::PathBuf;

/// Config command arguments.
#[derive(Debug, Clone)]
pub struct ConfigArgs {
    /// Get a specific key.
    pub get: Option<String>,
    /// Set a key-value pair (key=value).
    pub set: Option<String>,
    /// Show full config.
    pub show: bool,
    /// Validate configuration.
    pub validate: bool,
}

impl Default for ConfigArgs {
    fn default() -> Self {
        Self {
            get: None,
            set: None,
            show: false,
            validate: false,
        }
    }
}

/// Run the config command.
pub async fn run_config(args: ConfigArgs) -> Result<()> {
    let config_path = get_config_path();

    if args.validate {
        return validate_config(&config_path);
    }

    if let Some(key) = args.get {
        return get_config_value(&config_path, &key);
    }

    if let Some(kv) = args.set {
        return set_config_value(&config_path, &kv);
    }

    // Default: show full config
    show_config(&config_path)
}

/// Show the full configuration.
fn show_config(config_path: &PathBuf) -> Result<()> {
    if !config_path.exists() {
        ui::error(&format!("Config file not found: {}", config_path.display()));
        ui::info("Run 'openclaw onboard' to create configuration");
        return Ok(());
    }

    let content = std::fs::read_to_string(config_path)?;

    // Try to parse and pretty-print
    match serde_json::from_str::<serde_json::Value>(&content) {
        Ok(value) => {
            println!("{}", serde_json::to_string_pretty(&value)?);
        }
        Err(_) => {
            // JSON5 format, just print as-is
            println!("{}", content);
        }
    }

    Ok(())
}

/// Get a specific config value by path.
fn get_config_value(config_path: &PathBuf, key: &str) -> Result<()> {
    if !config_path.exists() {
        anyhow::bail!("Config file not found: {}", config_path.display());
    }

    let content = std::fs::read_to_string(config_path)?;
    let value: serde_json::Value = json5::from_str(&content)?;

    // Navigate the path (e.g., "gateway.port")
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = &value;

    for part in &parts {
        match current {
            serde_json::Value::Object(map) => {
                if let Some(v) = map.get(*part) {
                    current = v;
                } else {
                    ui::error(&format!("Key not found: {}", key));
                    return Ok(());
                }
            }
            serde_json::Value::Array(arr) => {
                if let Ok(idx) = part.parse::<usize>() {
                    if let Some(v) = arr.get(idx) {
                        current = v;
                    } else {
                        ui::error(&format!("Index out of bounds: {}", part));
                        return Ok(());
                    }
                } else {
                    ui::error(&format!("Invalid array index: {}", part));
                    return Ok(());
                }
            }
            _ => {
                ui::error(&format!("Cannot navigate into non-object: {}", part));
                return Ok(());
            }
        }
    }

    // Print the value
    match current {
        serde_json::Value::String(s) => println!("{}", s),
        serde_json::Value::Number(n) => println!("{}", n),
        serde_json::Value::Bool(b) => println!("{}", b),
        serde_json::Value::Null => println!("null"),
        _ => println!("{}", serde_json::to_string_pretty(current)?),
    }

    Ok(())
}

/// Set a config value.
fn set_config_value(config_path: &PathBuf, kv: &str) -> Result<()> {
    // Parse key=value
    let parts: Vec<&str> = kv.splitn(2, '=').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid format. Use: key=value or key.nested=value");
    }

    let key = parts[0];
    let new_value = parts[1];

    // Load existing config or create new
    let mut config: serde_json::Value = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        json5::from_str(&content)?
    } else {
        serde_json::json!({})
    };

    // Navigate and set the value
    let path_parts: Vec<&str> = key.split('.').collect();
    set_nested_value(&mut config, &path_parts, new_value)?;

    // Write back
    std::fs::create_dir_all(config_path.parent().unwrap())?;
    std::fs::write(config_path, serde_json::to_string_pretty(&config)?)?;

    ui::success(&format!("Set {} = {}", key, new_value));

    Ok(())
}

/// Set a nested value in a JSON object.
fn set_nested_value(
    root: &mut serde_json::Value,
    path: &[&str],
    value: &str,
) -> Result<()> {
    if path.is_empty() {
        return Ok(());
    }

    let mut current = root;

    // Navigate to parent
    for part in &path[..path.len() - 1] {
        if !current.is_object() {
            *current = serde_json::json!({});
        }

        let obj = current.as_object_mut().unwrap();
        if !obj.contains_key(*part) {
            obj.insert(part.to_string(), serde_json::json!({}));
        }
        current = obj.get_mut(*part).unwrap();
    }

    // Set the final value
    if !current.is_object() {
        *current = serde_json::json!({});
    }

    let obj = current.as_object_mut().unwrap();
    let final_key = path.last().unwrap();

    // Try to parse value as JSON, fall back to string
    let parsed_value = serde_json::from_str(value).unwrap_or_else(|_| {
        // Try as number
        if let Ok(n) = value.parse::<i64>() {
            return serde_json::Value::Number(n.into());
        }
        if let Ok(n) = value.parse::<f64>() {
            if let Some(num) = serde_json::Number::from_f64(n) {
                return serde_json::Value::Number(num);
            }
        }
        // Try as bool
        if value == "true" {
            return serde_json::Value::Bool(true);
        }
        if value == "false" {
            return serde_json::Value::Bool(false);
        }
        // Default to string
        serde_json::Value::String(value.to_string())
    });

    obj.insert(final_key.to_string(), parsed_value);

    Ok(())
}

/// Validate the configuration.
fn validate_config(config_path: &PathBuf) -> Result<()> {
    ui::header("Validating Configuration");

    if !config_path.exists() {
        ui::error(&format!("Config file not found: {}", config_path.display()));
        return Ok(());
    }

    let content = std::fs::read_to_string(config_path)?;

    // Check JSON5 syntax
    match json5::from_str::<serde_json::Value>(&content) {
        Ok(value) => {
            ui::success("Syntax: Valid JSON5");

            // Check required fields
            let mut warnings = Vec::new();

            if value.get("gateway").is_none() {
                warnings.push("Missing 'gateway' section");
            }

            if value.get("agents").is_none() {
                warnings.push("Missing 'agents' section");
            }

            if let Some(gateway) = value.get("gateway") {
                if gateway.get("port").is_none() {
                    warnings.push("Missing 'gateway.port'");
                }
            }

            if warnings.is_empty() {
                ui::success("Structure: All required fields present");
            } else {
                for w in warnings {
                    ui::warning(w);
                }
            }

            // Try loading with Config struct
            match openclaw_core::Config::load_default() {
                Ok(_) => {
                    ui::success("Schema: Configuration is valid");
                }
                Err(e) => {
                    ui::error(&format!("Schema error: {}", e));
                }
            }
        }
        Err(e) => {
            ui::error(&format!("Syntax error: {}", e));
        }
    }

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
