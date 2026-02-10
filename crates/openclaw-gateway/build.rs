//! Build script for openclaw-gateway.
//!
//! When the "ui" feature is enabled, this script builds the Vue UI
//! before compiling the gateway so that assets can be embedded.

use std::path::Path;
use std::process::Command;

fn main() {
    // Only build UI if the feature is enabled
    if std::env::var("CARGO_FEATURE_UI").is_err() {
        println!("cargo:warning=UI feature not enabled, skipping UI build");
        return;
    }

    // Tell Cargo to rerun if UI sources change
    println!("cargo:rerun-if-changed=../openclaw-ui/src");
    println!("cargo:rerun-if-changed=../openclaw-ui/public");
    println!("cargo:rerun-if-changed=../openclaw-ui/package.json");
    println!("cargo:rerun-if-changed=../openclaw-ui/vite.config.ts");
    println!("cargo:rerun-if-changed=../openclaw-ui/index.html");
    println!("cargo:rerun-if-changed=../openclaw-ui/tsconfig.json");

    // Check if we should skip UI build
    if std::env::var("SKIP_UI_BUILD").is_ok() {
        println!("cargo:warning=Skipping UI build (SKIP_UI_BUILD is set)");
        return;
    }

    let ui_dir = Path::new("../openclaw-ui");

    // Check if UI directory exists
    if !ui_dir.exists() {
        println!("cargo:warning=UI directory not found at ../openclaw-ui, skipping UI build");
        return;
    }

    // Determine npm command based on platform
    let npm_cmd = if cfg!(target_os = "windows") {
        "npm.cmd"
    } else {
        "npm"
    };

    // Check if node_modules exists, run npm install if not
    if !ui_dir.join("node_modules").exists() {
        println!("cargo:warning=Installing UI dependencies...");

        let status = Command::new(npm_cmd)
            .arg("install")
            .current_dir(ui_dir)
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("cargo:warning=UI dependencies installed successfully");
            }
            Ok(s) => {
                println!("cargo:warning=npm install failed with status: {}", s);
                // Don't fail the build, just warn
                return;
            }
            Err(e) => {
                println!("cargo:warning=Failed to run npm install: {}", e);
                println!("cargo:warning=Make sure Node.js and npm are installed");
                return;
            }
        }
    }

    // Check if dist directory exists and is up to date
    let dist_dir = ui_dir.join("dist");
    let should_build = if !dist_dir.exists() {
        true
    } else {
        // Check if any source files are newer than dist
        // For simplicity, we always rebuild in release mode
        std::env::var("PROFILE")
            .map(|p| p == "release")
            .unwrap_or(false)
            || !dist_dir.join("index.html").exists()
    };

    if should_build {
        println!("cargo:warning=Building UI...");

        let status = Command::new(npm_cmd)
            .args(["run", "build"])
            .current_dir(ui_dir)
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("cargo:warning=UI built successfully");
            }
            Ok(s) => {
                println!("cargo:warning=npm run build failed with status: {}", s);
                // Don't fail the build, just warn
            }
            Err(e) => {
                println!("cargo:warning=Failed to run npm build: {}", e);
            }
        }
    } else {
        println!("cargo:warning=UI dist directory exists, skipping build");
    }
}
